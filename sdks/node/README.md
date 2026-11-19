# Orchestrata native Node.js binding

This source package is not published and does not include the native engine or
its C header. Use the native library and header from the same engine build
(ABI version 1). A Node.js C toolchain is required.

To build from this repository, build `orchestrata-ffi`, then run
`npm run build` from this directory. The build looks in
`../../target/release` for the library and `../../crates/ffi/include`
for `orchestrata.h` by default.

To use a packed source package from a separate project, run `npm pack`
in this directory. From the consumer project, on Linux:

```sh
ORCHESTRATA_INCLUDE_DIR=/absolute/path/to/crates/ffi/include \
ORCHESTRATA_LIB_DIR=/absolute/path/to/target/release \
  npm install /absolute/path/to/orchestrata-native-0.1.0.tgz
LD_LIBRARY_PATH=/absolute/path/to/target/release node your-program.cjs
```

Use absolute, trusted artifact paths and the loader settings for your platform.
The library remains an external runtime dependency after installation.
`solveJson` takes a UTF-8 JSON string or Buffer and a bigint evaluation limit;
the result is lossless JSON text, not a parsed JavaScript object.

## Typed finite-domain API

`solve(problem, limit)` accepts the exported `Problem` type and a bigint budget. It returns `{ kind: 'solved', solution }` or `{ kind: 'infeasible' }`; errors throw `SolveError`. All seven finite-domain constraint types are exported from the package root. Use bigint for 64-bit quantities; indices and bounded scope counts use Number. The typed API requires lossless JSON support and was tested on Node 24.

```js
const { solve } = require('@orchestrata/native');
const result = solve({
  variables: [{ name: 'job', domain: [0n, 1n] }],
  constraints: [{ kind: 'prefer_value', variable: 0, value: 1n, softWeight: 1n }],
}, 2n);
if (result.kind !== 'solved' || result.solution.values[0] !== 1n) {
  throw new Error('Unexpected result');
}
```

Linux include/library paths may contain spaces and apostrophes. The build rejects control characters and Make/GYP-sensitive dollar, hash, backslash and double-quote characters. The package does not provide a recipe-package, routing or hosted endpoint.
