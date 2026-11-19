'use strict';

const native = require('./build/Release/orchestrata.node');
const { decodeResult } = require('./decode-result');
const { encodeProblem, requireBudget } = require('./encode-problem');
const { SolveError, nativeFailure } = require('./solve-error');

/**
 * @param {string | Buffer} problem Lossless UTF-8 JSON, not a JSON object
 * @param {bigint} maxEvaluations Unsigned 64-bit evaluation limit
 * @returns {string} Lossless JSON result, not parsed into JavaScript numbers
 */
function solveJson(problem, maxEvaluations) {
    if (typeof problem === 'string') {
        problem = Buffer.from(problem, 'utf8');
    } else if (!Buffer.isBuffer(problem)) {
        throw new TypeError('Problem must be a UTF-8 string or Buffer');
    }
    return native.solveJson(problem, maxEvaluations);
}

function solve(problem, maxEvaluations) {
    if (typeof JSON.rawJSON !== 'function'
        || JSON.parse('0', (_key, _value, context) => context?.source) !== '0') {
        throw new SolveError('unsupported_runtime', 'The typed API requires lossless JSON support in Node.js.');
    }

    const budget = requireBudget(maxEvaluations);
    const { json, domains } = encodeProblem(problem);
    let response;
    try {
        response = solveJson(json, budget);
    } catch (error) {
        throw nativeFailure(error);
    }

    return decodeResult(response, domains, budget);
}

module.exports = { SolveError, solve, solveJson };
