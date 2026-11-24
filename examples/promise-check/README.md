# Promise Check v1

Compare an incumbent assignment with explicit policies on one planning snapshot. The native entry point is `orchestrata_solver::promise_check::check_promises(&PromiseRequest)`. JSON is the reference client's transport; typed Rust consumers do not need JSON.

From the repository root:

```sh
cargo run --locked --manifest-path examples/promise-check/Cargo.toml -- check \
  < backend_tests/fixtures/promise-check/maintenance.json > decision.json
cargo run --locked --manifest-path examples/promise-check/Cargo.toml -- replay \
  < decision.json > replayed.json
cmp decision.json replayed.json
```

The version-1 request contains the snapshot, incumbent, confirmed assignments, policies and verification budget. Every policy maximizes assigned jobs, then minimizes one named metric. All policies use the same eligibility and job-count capacity. Estimates are nonnegative integers; their units are explicitly supplied and never converted. Option estimates and result totals follow the declared metric order; assignment vectors follow job order. Unknown fields and unsupported versions are rejected.

Confirmed assignments must match a feasible incumbent. They reserve capacity before optimization and remain fixed in every proposal. Unconfirmed work can move or become unassigned, including on objective ties: there is no minimum-change tie-break. Changed assignments and all unassigned job IDs are included without invented explanations.

The bundle embeds the complete request, incumbent accounting and per-policy evidence. Its scope is `SingleSnapshotCapacityAssignment`; its estimate basis is `SuppliedEstimatesNotObservedSavings`. Deltas are proposed minus incumbent and sum assigned work only. Compare coverage alongside totals. `ProvenOptimalUnderCommitments` refers only to coverage first and the selected metric second under the supplied fixed commitments. `VerifiedFeasibleOptimalityUnknown` means feasibility/accounting passed but the independent certificate exhausted its work budget. This is not infeasibility. Errors return no decision.

The synthetic seven-job example covers four jobs in the incumbent and five under either policy:

| Plan | Supplied cost (USD cents) | Supplied delay (minutes) |
| --- | ---: | ---: |
| Incumbent | 33000 | 73 |
| Minimize cost after coverage | 26000 | 205 |
| Minimize delay after coverage | 39000 | 108 |

The cost proposal moves two jobs and unassigns one previously assigned job; both proposals preserve the confirmed inspection. These are modeled trade-offs, not observed savings. Additive delay is not a route duration or a completion-time guarantee.

Bounds are 200 jobs/resources, eight metrics/policies, 16 short assumptions and 20 million certificate arc examinations per policy. The CLI accepts at most 1 MiB. Verification work does not bound solve time, parsing, allocation or wall-clock duration. There is no HTTP, storage, live-system freshness, operational write-back, routing, calendar scheduling or weekly/global optimality claim.

`replay` recomputes and compares the complete typed decision, including evidence. It rejects modified results; it is not a signature or authenticity proof. Snapshot IDs are caller labels, not content hashes. Preserve the bundle and the exact Engine source revision for reproducibility across software versions.

```sh
cargo test --locked --manifest-path examples/promise-check/Cargo.toml
cargo test --locked --manifest-path backend_tests/rust/Cargo.toml --test promise_check
```

A second deterministic fixture, `backend_tests/fixtures/promise-check/backlog.json`, contains 48 synthetic jobs, six crews and six confirmed assignments. A greedy feasible incumbent assigns 41 jobs; both policies assign 42. The cost policy changes modeled totals from 415400 cents / 1812 minutes to 362000 cents / 2126 minutes. The delay policy produces 450200 cents / 1499 minutes. These tests establish the stated fixture results, not production scale or realized savings.
