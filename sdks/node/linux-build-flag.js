'use strict';

const path = require('node:path');

const mode = process.argv[2];
const include = mode === 'include';
if (!include && mode !== 'library') {
    throw new Error('Expected include or library mode');
}

const configured = include
    ? process.env.ORCHESTRATA_INCLUDE_DIR || '../../crates/ffi/include'
    : process.env.ORCHESTRATA_LIB_DIR || '../../target/release';
const directory = path.resolve(configured);

// GYP inserts this argument into a Makefile recipe, not an argv array.
// Reject Make syntax and characters that GYP itself re-quotes.
if (/[\x00-\x1f\x7f$#\\"]/u.test(directory)) {
    throw new Error('The Linux FFI directory contains unsupported build-path characters');
}
const escaped = directory.replace(/'/gu, "'\\''");
process.stdout.write((include ? '-I' : '-L') + "'" + escaped + "'");
