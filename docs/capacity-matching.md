# Assign jobs within worker capacity

Ana can inspect equipment and check lift controls. Rami can inspect equipment.
Each can take one job. Assign the safety inspection to Rami so Ana can check the
lift controls. Leave the boiler repair unassigned because neither worker is
eligible for it.

Run the [Rust example](../crates/solver/examples/capacity_matching/main.rs) from
the engine repository root:

```sh
cargo run --locked -p orchestrata-solver --example capacity_matching
```

Expected output:

```text
Assigned 2 of 3 jobs.
Safety inspection: Rami
Lift control check: Ana
Boiler repair: unassigned
```

`match_capacity` takes a `CapacityMatchProblem`: a job-count limit for each
worker and a list of eligible worker indices for each job. Indices start at zero.
An empty eligibility list means no one is eligible. The result gives one worker
index or `None` per job, in the original job order. Invalid capacities and
repeated or unknown indices return an error, not a partial result.

The solver finds the **largest number of assignments** allowed by the input.
If several plans cover the same number of jobs, it does not rank them by
preference, priority, cost or fairness. Keep those trade-offs separate. The
model supports up to 200 jobs and 200 workers, with a capacity of 0 to 200 jobs
per worker. It does not model calendars, hours, travel or overlapping tasks.
This example uses the Rust API; it is not a request to the general native JSON
solver.

[Try an assignment in your browser](https://ilstarno.github.io/Orchestrata/try/).
