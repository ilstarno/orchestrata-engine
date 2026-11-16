'use strict';

const I64_MIN = -(1n << 63n);
const I64_MAX = (1n << 63n) - 1n;
const U64_MAX = (1n << 64n) - 1n;

function inRange(value, low, high) {
    return typeof value === 'bigint' && value >= low && value <= high;
}

module.exports = { I64_MIN, I64_MAX, U64_MAX, inRange };
