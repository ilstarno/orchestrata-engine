# Orchestrata for .NET 8

Build or provide a matching native library and reference `Orchestrata.csproj`.
The typed API uses the same native V1 ABI as the existing JSON methods.

```csharp
using Orchestrata;

using var solver = new Solver("/full/path/to/liborchestrata_ffi.so");
var problem = new Problem(
    [new Variable("job", [0, long.MaxValue])],
    [new PreferValue(0, long.MaxValue, 1)]);

switch (solver.Solve(problem, 2))
{
    case Solved solved:
        Console.WriteLine(solved.Solution.Values[0]); // 9223372036854775807
        break;
    case Infeasible _:
        Console.WriteLine("No assignment satisfies the hard constraints.");
        break;
}
```

The model supports `AllDifferent`, `NotEqualOffset`, `PreferValue`,
`CountValue`, `Precedence`, `NoOverlap`, and `ResourceCapacity`. Signed
values, offsets, weights, resource IDs and scores are `long`. Durations,
loads, capacities and evaluation counts/budgets are `ulong`. Zero-based
variable indices and scope counts are nonnegative `int` values. The encoder
writes exact JSON integers; the decoder checks numeric ranges and validates
the response shape, domains, hard score and budget before returning a result.

- `ArgumentException`: invalid managed request shape or range.
- `SolveException`: an engine rejection or execution error; `Kind` is the
  typed status, and `Status` remains the original numeric value. A
  `SolveStatus.SearchLimit` error is not proof of infeasibility.
- `SolveResponseException`: the native response did not match the typed V1
  result contract.
- `ObjectDisposedException`: the native library has been released.

`SolveJson` still accepts raw UTF-8 bytes or text and returns the raw JSON
response. See [the native smoke tests](../../backend_tests/csharp/README.md).
