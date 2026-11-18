# Matching with explicit omission penalties

The additive Rust API `capacity_matching::weighted::penalized::match_capacity_with_penalties` minimizes one objective:

`sum(cost of selected eligible assignments) + sum(unassigned cost of each omitted job)`.

All costs must use the same caller-defined unit, such as cents. They are explicit nonnegative u32 values; aggregate costs use u64. Jobs need one worker, and capacities count jobs. Eligibility and capacity are hard constraints. Unavailable workers can be supplied with zero capacity. No deadlines, duration overlap, concurrency, live routing or automatic penalty estimation is implemented.

Unlike `match_capacity_min_cost`, this API does not maximize coverage first. A feasible request can remain unassigned when paying its explicit penalty is cheaper than service. Equal-cost solutions have no hidden coverage or fairness preference. Replaying identical ordered inputs is deterministic; permutations are not promised to produce the same tied plan.

## Synthetic contact-center batch

```sh
cargo test --locked -p orchestrata-solver penalized
```

Six requests have supplied service and missed-service costs. The generalist can handle two requests, the specialist one, the language agent zero, and overflow one. The penalty model serves fraud, outage and compliance for 600 cents, leaves penalties of 13100 cents, and totals 13700 cents. Routine overflow service costs 900 cents against a 100-cent omission penalty, so unused overflow capacity is optimal. The unavailable language agent prevents its request being served regardless of penalty. Billing competes for scarce generalist/specialist capacity. Raising billing's penalty from 5000 to 9000 cents changes the optimum to serve billing instead of outage.

Coverage-first matching serves four requests with total service cost 1300 cents; its omitted requests incur 14000 cents, totaling 15300 cents under the penalty objective. These are different policies, not competing correctness claims. A caller must explicitly choose the policy; there is no default switch.

Tests independently enumerate 52488 small problems, verify exact objective values and feasibility, test residual rerouting, changed penalties, zero capacity, malformed eligibility, deterministic replay, the 200-job boundary with u32-maximum costs, and a dense 200-by-200 graph with an exact lower-bound certificate. This is bounded exact matching, not evidence of arbitrary production scale. Limits remain 200 jobs, 200 workers and capacity 200 each. The implementation reuses the existing residual-network min-cost flow solver and fully re-solves each input.

Mathematical XML v1 remains coverage-first and does not expose omission penalties. The explicit v2 interface is documented in [penalty-xml-v2.md](penalty-xml-v2.md). Existing v1 package identities, JSON solution shapes and loose XML behavior are unchanged. Validation of a package is not a claim of v1 or hosted execution support.
