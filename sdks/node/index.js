'use strict';

const native = require('./build/Release/orchestrata.node');

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

module.exports = { solveJson };
