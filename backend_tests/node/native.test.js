'use strict';

const assert = require('node:assert/strict');
const test = require('node:test');
const { solveJson } = require('../../sdks/node');

test('returns exact signed 64-bit values without converting them to Number', () => {
    const problem = '{"variables":[{"name":"x","domain":[9223372036854775807]}],"constraints":[]}';
    assert.match(solveJson(problem, 1n), /"values":\[9223372036854775807\]/);
});

test('returns null for proven infeasibility and distinctly reports a budget rejection', () => {
    const problem = '{"variables":[{"name":"x","domain":[0]}],"constraints":['
        + '{"kind":"not_equal_offset","left":0,"right":0,"offset":0,"hard_weight":1}]}';
    assert.equal(solveJson(problem, 1n), '{"solution":null}');
    assert.throws(() => solveJson(problem, 0n), { status: 2 });
    assert.throws(() => solveJson(problem, -1n), TypeError);
    assert.throws(() => solveJson(problem, 1), TypeError);
});
