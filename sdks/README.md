# Orchestrata native SDKs

The C#, Java, Python, Node.js, PHP, and Go bindings call the same native library. Build or supply a native library for the target operating system and architecture; the language packages do not bundle binaries.

## Build the shared library

From the repository root, with stable Rust and the platform linker:

```sh
cargo build --release -p orchestrata-ffi
```

The C contract is in [orchestrata.h](../crates/ffi/include/orchestrata.h). The release directory is normally `target/release`, containing `liborchestrata_ffi.so` on Linux, `liborchestrata_ffi.dylib` on macOS, or `orchestrata_ffi.dll` on Windows. Use the actual build target directory if Cargo has been configured differently. The bindings reject ABI versions other than 1.

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

Signed domain and score values are 64-bit; evaluation counts are unsigned 64-bit. The typed Node.js API uses `bigint` for 64-bit values and handles lossless wire encoding and decoding. Its legacy raw API requires JSON text or a Buffer, a `bigint` limit, and caller-owned lossless decoding. Standard `JSON.parse`/Number is not lossless for every result. Java passes a `long` limit as an unsigned bit pattern. PHP accepts nonnegative, signed PHP integers for the limit; its practical maximum is `PHP_INT_MAX`. C#, Java, and PHP return raw JSON rather than imposing a numeric representation. Node.js, Python, and Go provide lossless typed functions for all seven finite-domain rules.

## Use and smoke test

The examples below build from a checkout on Linux, with `lib` pointing to the native library directory:

```sh
lib="$PWD/target/release"
library="$lib/liborchestrata_ffi.so"
include="$PWD/crates/ffi/include"
```

**Python 3.10+.** Install from `sdks/python` or add that directory to `PYTHONPATH`. Construct `Solver(library)` and use `solve_problem(problem, limit)` with a typed `Problem`, `solve(mapping, limit)` with a dynamic mapping, or `solve_json(problem_bytes, limit)` for raw bytes. The typed classes include `AllDifferent`, `NotEqualOffset`, `PreferValue`, `CountValue`, `Precedence`, `NoOverlap`, and `ResourceCapacity`, using tuple fields and exact integer bounds.

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

**Go with cgo.** Use the [Go package](go/README.md) and call `SolveProblem(problem, limit)` for a fully typed request and result, or `Solve(problemJSON, limit)` for a raw request with a validated result. `SolveJSON` preserves the raw interface. Point the C compiler at the header and the linker at the native library.

```sh
(cd backend_tests/go && CGO_ENABLED=1 CGO_CFLAGS="-I$include" CGO_LDFLAGS="-L$lib" \
  LD_LIBRARY_PATH="$lib" go test -v ./...)
```

**Node.js.** Use a Node release with Node-API bigint support and install the platform C toolchain and `node-gyp`. The dynamic loader needs the native library on its search path. The typed API also requires `JSON.rawJSON` and the `JSON.parse` reviver's `context.source` (verified on Node 24).

```js
const { solve, SolveError } = require('./sdks/node');

try {
  const result = solve({
    variables: [{ name: 'job', domain: [0n, 9007199254740993n] }],
    constraints: [{ kind: 'prefer_value', variable: 0, value: 9007199254740993n, softWeight: 1n }],
  }, 2n);
  if (result.kind === 'solved') console.log(result.solution.values[0]);
} catch (error) {
  if (error instanceof SolveError) console.error(error.code);
  else throw error;
}
```

`solve(problem, maxEvaluations)` takes a typed object. Constraint kinds match the native names, with camelCase field names such as `hardWeight` and `minGap`. Domain values, offsets, durations, resource IDs and loads, weights, scores and the evaluation limit/count use `bigint`. Variable indices and scope counts use safe nonnegative `number` values. The result is either `{ kind: 'solved', solution }` or `{ kind: 'infeasible' }` for a proven infeasible problem. All numbers in the returned solution are `bigint`. Unknown input fields and malformed result data are rejected. The typed decoder expects the compact V1 output emitted by the native library and rejects duplicate fields or added whitespace.

Typed calls throw `SolveError` with one of these codes:

- `invalid_input`: invalid request shape, scalar range, or engine problem definition.
- `search_limit_exceeded`: not enough evaluations to search the full space; not an infeasibility result.
- `execution_failed`: the native call failed, for example because the winning score cannot fit in 64 bits.
- `invalid_response`: the native result did not match the expected result contract.
- `unsupported_runtime`: the Node runtime cannot encode and decode exact 64-bit integers.

`nativeStatus` is present when the native binding supplied a numeric status. `solveJson(problemJson, limitBigInt)` remains unchanged for callers handling raw JSON.

```sh
(cd sdks/node && ORCHESTRATA_LIB_DIR="$lib" npm run build)
LD_LIBRARY_PATH="$lib" node --test backend_tests/node/*.test.js
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
