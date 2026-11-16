# Orchestrata engine

Orchestrata is a Rust library for assigning integer values under hard rules and ranked preferences. Use the Rust API directly or call the native C ABI through the C#, Java, Python, Node.js, PHP, or Go bindings.

## Try a solve

Build the native library with stable Rust. On Linux, run the following from the repository root with Python 3.10 or later:

    cargo build --manifest-path backend/Cargo.toml --release --locked -p orchestrata-ffi

    PYTHONPATH=sdks/python python3 - <<'PY'
    from orchestrata import Solver

    problem = {
        "variables": [{"name": "job", "domain": [0, 1]}],
        "constraints": [
            {"kind": "prefer_value", "variable": 0, "value": 1, "soft_weight": 1}
        ],
    }
    solver = Solver("backend/target/release/liborchestrata_ffi.so")
    print(solver.solve(problem, 2)["values"])
    PY

This prints `[1]`. Exact search either returns a verified optimum, proves the submitted model infeasible, or rejects a search that exceeds the evaluation budget. Bounded local search can return a feasible plan but cannot prove optimality or infeasibility. The Rust API also compares verified plans and checks supplied pickup-and-delivery routes; it does not calculate travel times or construct routes.

See the [SDK guide](sdks/README.md) for the library name on your platform, native ABI details, and other languages. See [backend tests](backend_tests/README.md) for verification commands.
