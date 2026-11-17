# Engine tests

The Rust suites use a separate, non-publishable test workspace. Production builds use the root Cargo workspace and do not include these targets.

    cargo test --manifest-path backend_tests/rust/Cargo.toml --locked
    cargo test --manifest-path backend_tests/rust/Cargo.toml --locked --all-features

The C smoke source is `rust/ffi/header_smoke.c`. The [SDK guide](../sdks/README.md) gives the commands for Python, Go, Node.js, .NET, Java, and PHP. Their test sources are here rather than in their production packages.
