# Orchestrata native SDKs

The C#, Java, Python, Node.js, PHP, and Go bindings call the same native library. Build or supply a native library for the target operating system and architecture; the language packages do not bundle binaries.

## Build the shared library

From the repository root, with stable Rust and the platform linker:

```sh
cargo build --manifest-path backend/Cargo.toml --release -p orchestrata-ffi
```

The C contract is in [orchestrata.h](../backend/crates/ffi/include/orchestrata.h). The release directory is normally `backend/target/release`, containing `liborchestrata_ffi.so` on Linux, `liborchestrata_ffi.dylib` on macOS, or `orchestrata_ffi.dll` on Windows. Use the actual build target directory if Cargo has been configured differently. The bindings reject ABI versions other than 1.

## Contract

Pass a UTF-8 JSON problem and a maximum number of evaluations. A minimal problem is `{"variables":[{"name":"x","domain":[0,1]}],"constraints":[]}`. The exhaustive reference solver rejects a problem larger than the budget before evaluating anything. It returns an optimal, independently verified assignment, or `{"solution":null}` when it has proved infeasibility. No partial result is returned. The request is limited to 1 MiB and the C response to 2 MiB including its terminating NUL.

| Native status | Meaning |
| --- | --- |
| 0 | Result document returned |
| 1 | Invalid JSON or problem |
| 2 | Search space exceeds the requested evaluation limit |
| 3 | Execution error, including an unrepresentable winning score |
| 4 | Invalid ABI argument, including an empty or oversized request |

The caller borrows input for one synchronous call. Native callers release each output exactly once with `orchestrata_free_json_v1`; language bindings do this before returning a managed copy. The library has no process-wide solver state. Run long calls off a request or event-loop thread.

Signed domain and score values are 64-bit; evaluation counts are unsigned 64-bit. In Node.js give inputs as a JSON string or Buffer, set the limit with a `bigint` (for example, `1000n`), and decode the returned JSON with an integer-preserving parser. Standard `JSON.parse`/Number is not lossless for every result. Java passes a `long` limit as an unsigned bit pattern. PHP accepts nonnegative, signed PHP integers for the limit; its practical maximum is `PHP_INT_MAX`. C#, Java, Node.js, and PHP return raw JSON rather than imposing a numeric representation. Python and Go also provide lossless typed convenience methods.

## Use and smoke test

The examples below build from a checkout on Linux, with `lib` pointing to the native library directory:

```sh
lib="$PWD/backend/target/release"
library="$lib/liborchestrata_ffi.so"
include="$PWD/backend/crates/ffi/include"
```

**Python 3.10+.** Install from `sdks/python` or add that directory to `PYTHONPATH`. Construct `Solver(library)` and use `solve_problem(problem, limit)` with a typed `Problem`, `solve(mapping, limit)` with a dynamic mapping, or `solve_json(problem_bytes, limit)` for raw bytes. The typed classes include `AllDifferent`, `NotEqualOffset`, `PreferValue`, `CountValue`, and `ResourceCapacity`, using tuple fields and exact integer bounds.

```python
from orchestrata import PreferValue, Problem, Solver, Variable

solver = Solver("/absolute/path/to/liborchestrata_ffi.so")
problem = Problem(
    variables=(Variable("job", (0, 1)),),
    constraints=(PreferValue(variable=0, value=1, soft_weight=1),),
)
solution = solver.solve_problem(problem, max_evaluations=2)
print(solution.values if solution is not None else "No feasible assignment")
```

This prints `(1,)`. A typed solution exposes `values`, `score.hard`, `score.soft`, and `evaluated`. An explicitly proven infeasible request returns `None`. An exceeded evaluation limit raises `SolveError` with status 2. A malformed typed reply raises `ResultFormatError`; local type or range errors may raise `TypeError` or `ValueError` before execution. The two existing interfaces keep their original behavior.

```sh
PYTHONPATH="$PWD/sdks/python" ORCHESTRATA_LIBRARY="$library" \
  python -m unittest discover -s backend_tests/python -v
```

**Go with cgo.** Use the module in `sdks/go` and call `Solve(problemJSON, limit)` or `SolveJSON(problemJSON, limit)`. Point the C compiler at the header and the linker at the native library.

```sh
(cd backend_tests/go && CGO_ENABLED=1 CGO_CFLAGS="-I$include" CGO_LDFLAGS="-L$lib" \
  LD_LIBRARY_PATH="$lib" go test -v ./...)
```

**Node.js.** Use a Node release with Node-API bigint support and install the platform C toolchain and `node-gyp`. Call `solveJson(problemJson, limitBigInt)`. The dynamic loader needs the native library on its search path.

```sh
(cd sdks/node && ORCHESTRATA_LIB_DIR="$lib" npm run build)
LD_LIBRARY_PATH="$lib" node --test backend_tests/node/native.test.js
```

**.NET 8.** Reference `sdks/csharp/Orchestrata.csproj`. Construct `Solver(library)`, call `SolveJson(problemJson, limit)`, and dispose it after use.

```sh
dotnet run --project backend_tests/csharp/Smoke.csproj -- "$library"
```

**Java 22+ (64-bit).** Compile the sources under `sdks/java/src/main/java`, construct `io.orchestrata.Solver` with a library path, and close it after use. Enable native access for the binding module, or `ALL-UNNAMED` on the classpath.

```sh
mkdir -p backend_tests/java/build
javac -d backend_tests/java/build $(find sdks/java/src/main/java backend_tests/java/io -name '*.java')
java --enable-native-access=ALL-UNNAMED -cp backend_tests/java/build io.orchestrata.Smoke "$library"
```

**PHP 8.1+ with FFI.** Add `sdks/php` as a Composer path package. Construct `Orchestrata\Solver` with the library path and call `solveJson($problem, $limit)`. Only enable FFI where it is appropriate for the deployment.

```sh
php -d ffi.enable=1 backend_tests/php/smoke.php "$library"
```

Use the appropriate native loader path on other platforms (`DYLD_LIBRARY_PATH` for local macOS development or `PATH` on Windows where applicable). Release binaries, cross-platform test coverage, published language packages, and support guarantees are not provided by this source layout.
