# Orchestrata native SDKs

C#, Java, Python, Node.js, PHP and Go expose typed finite-domain problems and results through the same synchronous native ABI v1. All six support `AllDifferent`, `NotEqualOffset`, `PreferValue`, `CountValue`, `Precedence`, `NoOverlap` and `ResourceCapacity`. Variable references are zero-based indices. Construct language objects; the binding encodes the native request. Existing raw JSON methods remain available.

These are source packages for local builds. The commands below do not install a published registry release. Language packages do not bundle the native library or provide hosted execution, recipe-package execution, routing, events or cancellation APIs. Use the separate Rust/recipe CLI contracts for those supported capabilities; the typed SDK examples do not establish package or hosted support.

## Build the shared library

From the repository root, with stable Rust and the platform linker:

```sh
cargo build --locked --release -p orchestrata-ffi
lib="$PWD/target/release"
library="$lib/liborchestrata_ffi.so"
include="$PWD/crates/ffi/include"
```

These examples use Linux. On macOS the library is `liborchestrata_ffi.dylib`; on Windows it is `orchestrata_ffi.dll`. Use the actual directory if Cargo has a custom target directory. Supply a matching native library for your operating system and architecture. The [C contract](../crates/ffi/include/orchestrata.h) remains ABI version 1; older ABI-v1 builds can still reject newer constraint kinds.

## Typed entrypoints and results

| Language | Typed call | Feasible result | Proven infeasibility |
| --- | --- | --- | --- |
| Python | `Solver.solve_problem(Problem, max_evaluations)` | `Solution` | `None` |
| Go | `SolveProblem(Problem, limit)` | `*Solution, nil` | `nil, nil` |
| Node.js | `solve(problem, limitBigInt)` | `{ kind: 'solved', solution }` | `{ kind: 'infeasible' }` |
| C# | `Solver.Solve(Problem, limit)` | `Solved` | `Infeasible` |
| Java | `Solver.solve(Problem, limitBits)` | `Optional<Solution>` containing a value | Empty `Optional` |
| PHP | `Solver->solve(Problem, limit)` | `Solution` | `null` |

The exact finite-domain solver rejects a search space larger than the caller's evaluation budget before searching. Budget rejection is an error, not infeasibility or a partial solution. A completed feasible search returns an exact optimum for the submitted supported model. This guarantee does not apply to the separate heuristic routing algorithms.

| Native status | Meaning |
| --- | --- |
| 0 | Result returned |
| 1 | Invalid JSON or problem |
| 2 | Search space exceeds the evaluation budget |
| 3 | Execution error, including an unrepresentable winning score |
| 4 | Invalid ABI argument, including an empty or oversized request |

The request limit is 1 MiB; the response limit is 2 MiB including its terminating NUL. Bindings free native results before returning managed copies. Calls are synchronous; run long calls off a request or event-loop thread. Local shape/range errors and response-decoding failures are distinct from native statuses; see each binding's types and tests.

Signed domains and scores use 64-bit integers. Python uses exact integers, Go uses `int64`/`uint64`, C# uses `long`/`ulong`, and Node uses `bigint` for 64-bit quantities. Java uses `long` for signed values, `BigInteger` for unsigned rule quantities and result counts, and passes the `long` budget as unsigned bits (`-1L` means the maximum `u64`). PHP input quantities and budgets are limited to nonnegative signed PHP integers where unsigned values are required (`PHP_INT_MAX` maximum); wide unsigned result counts are decimal strings. PHP requires a 64-bit runtime for typed solves.

## Python 3.10+

```sh
python3 -m pip install ./sdks/python
PYTHONPATH="$PWD/sdks/python" ORCHESTRATA_LIBRARY="$library" \
  python3 -m unittest discover -s backend_tests/python -v
```

```python
from orchestrata import PreferValue, Problem, Solver, Variable

solver = Solver("/absolute/path/to/liborchestrata_ffi.so")
problem = Problem((Variable("job", (0, 1)),), (PreferValue(0, 1, 1),))
result = solver.solve_problem(problem, max_evaluations=2)
assert result is not None and result.values == (1,)
```

See the [Python usage guide](python/README.md). `solve` retains its dynamic-mapping interface and `solve_json` returns raw bytes.

## Go with cgo

Use the local module `sdks/go` with Go 1.20+ and a C compiler. Its module path is `github.com/ilstarno/Orchestrata/sdks/go`; the [test module](../backend_tests/go/go.mod) demonstrates a local `replace` directive. This is a local checkout dependency, not a new module-version publication.

```sh
(cd backend_tests/go && CGO_ENABLED=1 CGO_CFLAGS="-I$include" CGO_LDFLAGS="-L$lib" \
  LD_LIBRARY_PATH="$lib" go test -v ./...)
```

Use `SolveProblem` with typed models; `Solve` returns a typed result from JSON input and `SolveJSON` preserves raw bytes. See the [scheduling examples](../backend_tests/go/scheduling_test.go).

## Node.js

The typed API requires lossless JSON support (`JSON.rawJSON` and the parse reviver's `context.source`); validation used Node 24. A C toolchain and `node-gyp` are required. The legacy `solveJson` takes a UTF-8 string or Buffer and a bigint budget, and returns raw JSON text. Avoid ordinary `JSON.parse` for its 64-bit numbers.

```sh
(cd sdks/node && ORCHESTRATA_INCLUDE_DIR="$include" ORCHESTRATA_LIB_DIR="$lib" npm run build)
LD_LIBRARY_PATH="$lib" node --test backend_tests/node/*.test.js
sh backend_tests/node/standalone-install.test.sh "$PWD/sdks/node" "$include" "$lib"
```

The last command packs and installs into a clean consumer, then executes the native library. See the [Node package guide](node/README.md) and [typed examples](../backend_tests/node/typed.test.js).

## C# / .NET 8

Reference `sdks/csharp/Orchestrata.csproj`, or build a local NuGet package:

```sh
dotnet pack sdks/csharp/Orchestrata.csproj -c Release -o /tmp/orchestrata-nuget
cc -shared -fPIC -I"$include" backend_tests/csharp/malformed_native.c -o /tmp/orchestrata-malformed.so
dotnet run --project backend_tests/csharp/Smoke.csproj -- "$library" /tmp/orchestrata-malformed.so
```

A clean consumer can reference `Orchestrata` version `1.0.0` from `/tmp/orchestrata-nuget` as a local NuGet source; no registry publication is implied. Use `Solver` in a `using` scope and call `Solve` with typed objects. `SolveJson` retains raw UTF-8/text support. See [typed native tests](../backend_tests/csharp/TypedTests.cs) for all seven rules and error cases.

## Java 22+ (64-bit)

```sh
sh sdks/java/build-jar.sh /tmp/orchestrata.jar
sh backend_tests/java/test-packaged.sh "$library"
```

The packaged test compiles its consumer against only the JAR and executes the native library. Use `Solver` in try-with-resources; enable native access for the binding module, or `ALL-UNNAMED` on the classpath. See the [Java guide](java/README.md). `solveJson` remains available for lossless raw text.

## PHP 8.1+ with FFI

Add `sdks/php` as a local Composer path repository with mirroring (`"symlink": false`), then require its package `orchestrata/native`. In a separate consumer, substitute the absolute checkout path in this `composer.json`:

```json
{
  "require": {"orchestrata/native": "dev-main"},
  "repositories": [
    {"type": "path", "url": "/absolute/path/to/engine/sdks/php", "options": {"symlink": false, "versions": {"orchestrata/native": "dev-main"}}},
    {"packagist.org": false}
  ]
}
```

Run `composer install --no-plugins --no-scripts`, then execute the installed package's typed tests from the Engine checkout:

```sh
php -d ffi.enable=1 backend_tests/php/typed_constraints.php /absolute/path/to/consumer/vendor/autoload.php "$library"
php -d ffi.enable=1 backend_tests/php/smoke.php "$library"
```

Construct typed objects and call `Orchestrata\Solver->solve`; see the [typed example](../backend_tests/php/typed_solve.php). `solveJson` preserves raw JSON. Enable FFI only where permitted by your runtime environment.

Native package-consumer checks have run on Linux. These instructions do not establish cross-platform binary coverage, registry releases or deployment support. Use the loader settings appropriate to your platform and trusted native artifacts.
