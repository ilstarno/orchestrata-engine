# .NET native smoke tests

Run from the repository root with .NET 8 and the native Rust build:

```sh
cargo build --locked -p orchestrata-ffi
dotnet run --project backend_tests/csharp/Smoke.csproj -- "$PWD/target/debug/liborchestrata_ffi.so"
```

On Linux, include a test-only alternate library to exercise malformed responses:

```sh
cc -std=c11 -shared -fPIC -Wall -Wextra -Werror -Icrates/ffi/include \
  backend_tests/csharp/malformed_native.c -o /tmp/libmalformed_orchestrata.so
dotnet run --project backend_tests/csharp/Smoke.csproj -- \
  "$PWD/target/debug/liborchestrata_ffi.so" /tmp/libmalformed_orchestrata.so
```

The second library is only a test fixture; it is not part of the SDK.
