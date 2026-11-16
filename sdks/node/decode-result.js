'use strict';

const { I64_MIN, I64_MAX, U64_MAX, inRange } = require('./integers');
const { SolveError } = require('./solve-error');

function invalidResponse() {
    return new SolveError('invalid_response', 'The native solver returned an invalid V1 result.');
}

function shape(value, fields) {
    return value !== null && typeof value === 'object' && !Array.isArray(value)
        && Object.keys(value).length === fields.length && fields.every(field => Object.hasOwn(value, field));
}

function decodeResult(json, domains, budget) {
    if (typeof json !== 'string') {
        throw invalidResponse();
    }

    let document;
    try {
        document = JSON.parse(json, (_key, value, context) => {
            if (typeof value !== 'number') {
                return value;
            }
            if (typeof context?.source !== 'string' || !/^-?(?:0|[1-9][0-9]{0,19})$/.test(context.source)) {
                throw invalidResponse();
            }
            return BigInt(context.source);
        });
    } catch {
        throw invalidResponse();
    }

    // Rust's V1 response uses compact JSON. A re-encode rejects duplicate
    // object members and noncanonical integers that JSON.parse would collapse.
    const compact = JSON.stringify(document, (_key, value) =>
        typeof value === 'bigint' ? JSON.rawJSON(value.toString()) : value);
    if (compact !== json || !shape(document, ['solution'])) {
        throw invalidResponse();
    }
    if (document.solution === null) {
        return { kind: 'infeasible' };
    }

    const solution = document.solution;
    if (!shape(solution, ['values', 'score', 'evaluated'])
        || !Array.isArray(solution.values) || solution.values.length !== domains.length
        || !shape(solution.score, ['hard', 'soft']) || solution.score.hard !== 0n
        || !inRange(solution.score.soft, I64_MIN, 0n)
        || !inRange(solution.evaluated, 1n, U64_MAX) || solution.evaluated > budget
        || !solution.values.every((value, index) => inRange(value, I64_MIN, I64_MAX) && domains[index].includes(value))) {
        throw invalidResponse();
    }

    return { kind: 'solved', solution };
}

module.exports = { decodeResult };
