# Orchestrata for Python

The Python bindings require Python 3.10 or later and a separately built
Orchestrata native library. The wheel contains the bindings, not the native
library.

From the repository root, build and install into your Python environment:

```sh
cargo build --locked --release -p orchestrata-ffi
python -m pip install ./sdks/python
```

On Linux, run:

```python
from orchestrata import PreferValue, Problem, Solver, Variable

solver = Solver("target/release/liborchestrata_ffi.so")
problem = Problem((Variable("job", (0, 1)),), (PreferValue(0, 1, 1),))
result = solver.solve_problem(problem, max_evaluations=2)
assert result is not None
print(result.values)  # (1,)
```

On macOS use `liborchestrata_ffi.dylib`; on Windows use
`orchestrata_ffi.dll`. Pass a path to the built library, not the path to the
wheel. Use an absolute path if your script runs outside the repository.

`None` means the submitted model has no feasible assignment. An exhausted
evaluation budget raises `SolveError` with status `2`; invalid native input
raises `SolveError` with status `1` or `4`.
