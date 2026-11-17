# Browser engine demonstration

The WebAssembly module runs locally. It imports no host functions. Serve `public/try-engine.wasm` with content type `application/wasm`, alongside `try-engine.mjs` and `try-engine-licenses.txt`. Instantiate via `const engine = await createEngine(bytes)` from the module.

- `engine.matchCapacity({ worker_capacities, eligible_workers_by_job })` returns `{ workers_by_job, assigned }`. Each entry is an eligible worker’s zero-based index or `null`. It finds the maximum number of assigned independent jobs within eligibility and per-worker job-count limits, up to 200 jobs and 200 workers in 256 KiB. It does not model travel, calendars, preferences or general scheduling rules.
- `engine.solveProblem(problem, budget)` runs the native exhaustive planning contract for small examples (128 KiB and 4,096 complete assignments maximum). It returns `{ solution }`, including `null` only after a proof of infeasibility. Invalid input, budget exhaustion and engine failure throw distinct errors. Use `solveRequest(json, budget)` for exact signed 64-bit JSON output; the object helper rejects unsafe JavaScript numeric results.

## Build and verify

Run these commands from this example directory. Use stable Rust, the wasm32 target, and the checked-in lockfile:

```sh
rustup target add wasm32-unknown-unknown
cargo build --locked --release --target wasm32-unknown-unknown
cp target/wasm32-unknown-unknown/release/orchestrata_browser_demo.wasm public/try-engine.wasm
node --test tests/parity.test.mjs
```

Set `ORCHESTRATA_FIXTURES_DIR` to the public site fixtures directory to test the small native replay outputs as well. All limits refer to a single local request. No data is sent to a server by either adapter operation.
