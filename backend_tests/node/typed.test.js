'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { SolveError, solve } = require('../../sdks/node');
const { decodeResult } = require('../../sdks/node/decode-result');

const maxI64 = (1n << 63n) - 1n;
const minI64 = -(1n << 63n);
const maxU64 = (1n << 64n) - 1n;

function variable(name, ...domain) {
    return { name, domain };
}

test('round-trips signed domain and score boundaries as bigint through the real ABI', () => {
    const result = solve({
        variables: [variable('early', minI64), variable('late', maxI64)],
        constraints: [
            { kind: 'prefer_value', variable: 0, value: 0n, softWeight: maxI64 },
            { kind: 'prefer_value', variable: 0, value: 0n, softWeight: 1n },
        ],
    }, maxU64);
    assert.deepEqual(result, {
        kind: 'solved',
        solution: { values: [minI64, maxI64], evaluated: 1n, score: { hard: 0n, soft: minI64 } },
    });
});

test('encodes all current finite-domain constraint variants', () => {
    const result = solve({
        variables: [variable('start', 0n), variable('next', 10n), variable('resource', 2n)],
        constraints: [
            { kind: 'all_different', variables: [0, 1], hardWeight: 1n },
            { kind: 'not_equal_offset', left: 0, right: 1, offset: 0n, hardWeight: 1n },
            { kind: 'prefer_value', variable: 1, value: 10n, softWeight: 1n },
            { kind: 'count_value', variables: [2], value: 2n, min: 1, max: 1, weight: 1n, tier: 'hard' },
            { kind: 'precedence', before: 0, after: 1, minGap: 10n },
            { kind: 'no_overlap', intervals: [{ start: 0, duration: 10n }, { start: 1, duration: 1n }] },
            { kind: 'resource_capacity', items: [{ variable: 2, load: maxU64 }], limits: [{ resource: 2n, capacity: maxU64 }] },
        ],
    }, 1n);
    assert.deepEqual(result, {
        kind: 'solved',
        solution: { values: [0n, 10n, 2n], evaluated: 1n, score: { hard: 0n, soft: 0n } },
    });
});

test('distinguishes proven infeasibility, native budget rejection, and engine failure', () => {
    const problem = {
        variables: [variable('job', 0n)],
        constraints: [{ kind: 'not_equal_offset', left: 0, right: 0, offset: 0n, hardWeight: 1n }],
    };
    assert.deepEqual(solve(problem, 1n), { kind: 'infeasible' });
    assert.throws(() => solve(problem, 0n), { code: 'search_limit_exceeded', nativeStatus: 2 });

    const overflow = {
        variables: problem.variables,
        constraints: [
            { kind: 'prefer_value', variable: 0, value: 1n, softWeight: maxI64 },
            { kind: 'prefer_value', variable: 0, value: 1n, softWeight: 2n },
        ],
    };
    assert.throws(() => solve(overflow, 1n), { code: 'execution_failed', nativeStatus: 3 });

    const badRule = { variables: problem.variables, constraints: [
        { kind: 'precedence', before: 0, after: 0, minGap: 0n },
    ] };
    assert.throws(() => solve(badRule, 1n), { code: 'invalid_input', nativeStatus: 1 });
});

test('rejects imprecise or malformed request data before native execution', () => {
    const empty = { variables: [], constraints: [] };
    for (const [problem, budget] of [
        [empty, 1], [empty, -1n], [empty, maxU64 + 1n],
        [{ variables: [variable('unsafe', 1)], constraints: [] }, 1n],
        [{ variables: [variable('overflow', maxI64 + 1n)], constraints: [] }, 1n],
        [{ variables: [variable('empty')], constraints: [] }, 1n],
        [{ variables: new Array(1), constraints: [] }, 1n],
        [{ ...empty, unknown: true }, 1n],
        [{ ...empty, constraints: [{ kind: 'not_a_rule' }] }, 1n],
        [{ ...empty, constraints: [{ kind: 'prefer_value', variable: 0, value: 0n, softWeight: 1n }] }, 1n],
    ]) {
        assert.throws(() => solve(problem, budget), error =>
            error instanceof SolveError && error.code === 'invalid_input' && error.nativeStatus === undefined);
    }
});

test('rejects corrupt or out-of-model responses without Number conversion', () => {
    const safe = '{"solution":{"values":[9223372036854775807],"score":{"hard":0,"soft":0},"evaluated":1}}';
    assert.equal(decodeResult(safe, [[maxI64]], 1n).solution.values[0], maxI64);
    for (const json of [
        '{', '{}', '[]', '{"solution":false}', ` ${safe}`,
        safe.replace(/}$/, ',"solution":null}'),
        safe.replace('9223372036854775807', '9223372036854775808'),
        safe.replace('9223372036854775807', '0'),
        safe.replace('"hard":0', '"hard":-1'),
        safe.replace('"evaluated":1', '"evaluated":1e0'),
        safe.replace('"evaluated":1', '"evaluated":2'),
    ]) {
        assert.throws(() => decodeResult(json, [[maxI64]], 1n), { code: 'invalid_response' });
    }
});

test('all seven wire variants reach the engine and affect solver behavior', () => {
    const variables = [variable('start', 0n), variable('next', 10n), variable('resource', 2n)];
    const hardRules = [
        { kind: 'all_different', variables: [0, 0], hardWeight: 1n },
        { kind: 'not_equal_offset', left: 0, right: 1, offset: -10n, hardWeight: 1n },
        { kind: 'count_value', variables: [2], value: 3n, min: 1, max: 1, weight: 1n, tier: 'hard' },
        { kind: 'precedence', before: 0, after: 1, minGap: 11n },
        { kind: 'no_overlap', intervals: [{ start: 0, duration: 11n }, { start: 1, duration: 1n }] },
        { kind: 'resource_capacity', items: [{ variable: 2, load: maxU64 }], limits: [{ resource: 2n, capacity: maxU64 - 1n }] },
    ];
    for (const constraint of hardRules) {
        assert.deepEqual(solve({ variables, constraints: [constraint] }, 1n), { kind: 'infeasible' }, constraint.kind);
    }

    const preferred = solve({
        variables,
        constraints: [{ kind: 'prefer_value', variable: 1, value: 11n, softWeight: 3n }],
    }, 1n);
    assert.equal(preferred.solution.score.soft, -3n);
});
