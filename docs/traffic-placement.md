# Review DNS steering and backend placement

These are separate Rust planning functions. They take supplied health and capacity snapshots and return advisory policies. Neither performs lookups, sends requests, checks hosts, changes a zone or load balancer, or estimates current traffic. No REST, OpenAPI, FFI, or SDK binding is included.

## Propose new-lookup DNS weights

Give `plan_dns_steering` an ordered list of approved regions, a fixed evaluation time, and endpoint rows. The solver chooses the first approved region with an admitted, explicitly healthy endpoint that has a current check and positive headroom. Targets in other regions have zero weight, including usable endpoints in later fallback regions. When no approved region qualifies, the policy has no selected region and every weight is zero.

Headroom is the configured relative capacity less commitments and reserve, floored at zero. Weights in the selected region sum to 1,000 relative units. Whole weights use largest-remainder rounding with ascending opaque endpoint ID for ties. Small targets can round down to zero. This is a strict region fallback policy, not capacity spillover between healthy regions; it does not predict requests, model a resolver cache, or assert that any provider accepts the returned weight scale. Review provider-specific conversion and published DNS rules separately.

```rust
use orchestrata_solver::traffic_placement::{
    CapacityEnvelope, DnsEndpoint, DnsRequest, Health, HealthSnapshot, StalenessPolicy,
    plan_dns_steering,
};

let check = HealthSnapshot {
    status: Health::Healthy,
    checked_at_seconds: Some(995),
};
let endpoints = [
    DnsEndpoint {
        id: "edge-west-a", region: "west", admitted: true, observation: check,
        capacity: CapacityEnvelope { configured_limit: 100, committed: 20, reserve: 20 },
    },
    DnsEndpoint {
        id: "edge-west-b", region: "west", admitted: true, observation: check,
        capacity: CapacityEnvelope { configured_limit: 60, committed: 10, reserve: 10 },
    },
];
let snapshot = StalenessPolicy { evaluated_at_seconds: 1000, maximum_age_seconds: 10 };
let plan = plan_dns_steering(
    DnsRequest { approved_regions: &["west", "east"], staleness: snapshot },
    &endpoints,
).unwrap();
assert_eq!(plan.selected_region, Some("west"));
assert_eq!(plan.endpoint_weights[0].weight, 600);
assert_eq!(plan.endpoint_weights[1].weight, 400);
```

## Allocate a defined interval of additional load

Give `plan_backend_allocation` one pool, an additional demand count, and backend rows in a consistent capacity unit/interval. Out-of-pool rows cannot contribute. A backend's headroom is its configured limit less commitments and reserve. The plan serves at most that headroom and exposes the rest as unserved, without stealing reserved capacity. It first maximizes served demand, then apportions the whole served units proportionally to headroom with deterministic largest-remainder rounding. An allocation stays below or equal to each backend's headroom. This is not a live assignment policy for existing connections, sticky sessions, retries or queues.

```rust
use orchestrata_solver::traffic_placement::{
    AllocationRequest, Backend, CapacityEnvelope, Health, HealthSnapshot, StalenessPolicy,
    plan_backend_allocation,
};

let plan = plan_backend_allocation(
    AllocationRequest {
        pool: "web", demand: 90,
        staleness: StalenessPolicy { evaluated_at_seconds: 1000, maximum_age_seconds: 15 },
    },
    &[Backend {
        id: "web-a", pool: "web", admitted: true,
        observation: HealthSnapshot { status: Health::Healthy, checked_at_seconds: Some(990) },
        capacity: CapacityEnvelope { configured_limit: 100, committed: 30, reserve: 20 },
    }],
).unwrap();
assert_eq!((plan.served, plan.unserved), (50, 40));
```

## Input and diagnostics

- Up to 256 targets in either function and up to 16 approved regions for DNS. Identifiers are opaque ASCII codes (1–64 bytes; letters, digits, `-`, `_`, `.`, and `:`). Repeated target IDs or regions are rejected rather than folded.
- A check exactly at the maximum age is accepted. A healthy state without a timestamp, an observation outside the age limit, unknown health and unhealthy status cannot supply usable capacity. Future timestamps reject the whole input, even if the row is outside the chosen pool or region.
- Output is sorted by stable ID. Every target receives a disposition, such as excluded scope, no headroom, or old/missing observation. Capacity math uses integers; the aggregate backend capacity uses `u128`.

| Code | Meaning |
| --- | --- |
| TRF-001 | The placement input exceeds the allowed dimensions. |
| TRF-002 | An endpoint, region, or pool key is not a valid opaque identifier. |
| TRF-003 | Target identifiers must be unique within the supplied snapshot. |
| TRF-004 | An approved steering region appears more than once. |
| TRF-005 | A health check occurs after the snapshot evaluation time. |
| TRF-006 | A steering policy must have at least one approved region. |
