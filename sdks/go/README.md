# Orchestrata Go binding

Build the matching native library, install Go 1.20 or newer and a C compiler,
and enable cgo. The package does not include a native binary.

```go
problem := orchestrata.Problem{
    Variables: []orchestrata.Variable{
        {Name: "room_a_start", Domain: []int64{0}},
        {Name: "room_b_start", Domain: []int64{20, 30}},
    },
    Constraints: []orchestrata.Constraint{
        orchestrata.NoOverlap{Intervals: []orchestrata.ScheduledInterval{
            {Start: 0, Duration: 30},
            {Start: 1, Duration: 15},
        }},
    },
}
solution, err := orchestrata.SolveProblem(problem, 2)
// solution.Values is []int64{0, 30}; nil with no error means proved infeasible.
```

Pass rule values (not pointers). Each rule uses zero-based indices into the
problem's variables. Domain values, resource IDs, and scores are signed
64-bit integers. Evaluation limits and counts are unsigned 64-bit integers.
No client-side floating-point conversion is performed.

| Rule type | Inputs and meaning |
| --- | --- |
| `AllDifferent` | Variable indices and a positive hard weight. Penalizes repeated values. |
| `NotEqualOffset` | Two variable indices, a signed offset, and a positive hard weight. Penalizes pairs whose difference equals the offset. |
| `PreferValue` | One variable index, a signed value, and a positive soft weight. |
| `CountValue` | Distinct indices, a signed value, inclusive min/max counts, positive weight, and `CountValueHard` or `CountValueSoft`. |
| `Precedence` | Distinct before/after variable indices and an unsigned minimum gap in the same unit as their domains. |
| `NoOverlap` | At least two distinct start references and unsigned durations in the same unit as their domains. Intervals are half-open. Zero duration is empty. |
| `ResourceCapacity` | Each item contributes an unsigned load to the resource selected by a variable; each signed resource ID has an inclusive unsigned capacity. Model one consistent dimension and unit at a time. |

- `SolveProblem` encodes the typed request and returns a typed, exact result.
  The decoder rejects malformed responses and values outside the request
  domains. Native validation remains authoritative for the model.
- `Solve` takes UTF-8 JSON and returns a typed result with strict response
  decoding. `SolveJSON` keeps the unchanged raw UTF-8 input/output contract.
- `SolveError.Status` reports native failures. `SearchLimit` does not mean
  infeasible. Only an explicit `{"solution":null}` indicates proven infeasibility.
  Encoding and response-format errors are separate from native statuses.

The ABI permits a 1 MiB request and a 2 MiB response including its terminating
NUL. Calls are synchronous and use the finite-domain exact solver; routing and
bounded local search are not supported by this binding. Use an engine build
that includes the rules you send. Older native builds may reject newer rules
even when the library reports ABI version 1.
