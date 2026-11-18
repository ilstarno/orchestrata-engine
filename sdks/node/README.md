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
