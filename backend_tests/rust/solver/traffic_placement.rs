// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use orchestrata_solver::traffic_placement::{
    AllocationRequest, Backend, BackendDisposition, CapacityEnvelope, DNS_WEIGHT_SCALE,
    DnsDisposition, DnsEndpoint, DnsRequest, Health, HealthSnapshot, StalenessPolicy, TrafficError,
    UnusableReason, plan_backend_allocation, plan_dns_steering,
};

const NOW: StalenessPolicy = StalenessPolicy {
    evaluated_at_seconds: 100,
    maximum_age_seconds: 30,
};
const GOOD: HealthSnapshot = HealthSnapshot {
    status: Health::Healthy,
    checked_at_seconds: Some(95),
};

fn capacity(value: u64) -> CapacityEnvelope {
    CapacityEnvelope {
        configured_limit: value,
        committed: 0,
        reserve: 0,
    }
}

fn endpoint<'a>(id: &'a str, region: &'a str, headroom: u64) -> DnsEndpoint<'a> {
    DnsEndpoint {
        id,
        region,
        admitted: true,
        observation: GOOD,
        capacity: capacity(headroom),
    }
}

fn backend<'a>(id: &'a str, pool: &'a str, headroom: u64) -> Backend<'a> {
    Backend {
        id,
        pool,
        admitted: true,
        observation: GOOD,
        capacity: capacity(headroom),
    }
}

fn placement(demand: u64) -> AllocationRequest<'static> {
    AllocationRequest {
        pool: "web",
        demand,
        staleness: NOW,
    }
}

#[test]
fn chooses_first_usable_region_and_apportions_its_weight_stably() {
    let regions = ["west", "east"];
    let request = DnsRequest {
        approved_regions: &regions,
        staleness: NOW,
    };
    let input = [
        endpoint("west-c", "west", 50),
        endpoint("east-d", "east", u64::MAX),
        endpoint("west-a", "west", 50),
        endpoint("outside", "outside", u64::MAX),
        endpoint("west-b", "west", 50),
    ];
    let policy = plan_dns_steering(request, &input).unwrap();
    assert_eq!(policy.selected_region, Some("west"));
    assert_eq!(
        policy
            .endpoint_weights
            .iter()
            .map(|e| (e.endpoint_id, e.weight, e.disposition))
            .collect::<Vec<_>>(),
        [
            ("east-d", 0, DnsDisposition::EarlierRegionAvailable),
            ("outside", 0, DnsDisposition::RegionNotApproved),
            ("west-a", 334, DnsDisposition::WeightProposed),
            ("west-b", 333, DnsDisposition::WeightProposed),
            ("west-c", 333, DnsDisposition::WeightProposed),
        ]
    );
    let mut permuted = input;
    permuted.reverse();
    assert_eq!(policy, plan_dns_steering(request, &permuted).unwrap());
}

#[test]
fn never_leaks_dns_weights_to_stale_unknown_or_forbidden_endpoints() {
    let mut west_a = endpoint("west-a", "west", 10);
    west_a.observation.checked_at_seconds = Some(69);
    let west_b = DnsEndpoint {
        observation: HealthSnapshot {
            status: Health::Unknown,
            checked_at_seconds: Some(95),
        },
        ..endpoint("west-b", "west", 10)
    };
    let east = DnsEndpoint {
        observation: HealthSnapshot {
            checked_at_seconds: Some(70),
            ..GOOD
        },
        ..endpoint("east", "east", 10)
    };
    let request = DnsRequest {
        approved_regions: &["west", "east"],
        staleness: NOW,
    };
    let forbidden = endpoint("north", "north", 999);
    let policy = plan_dns_steering(request, &[west_a, west_b, east, forbidden]).unwrap();
    assert_eq!(policy.selected_region, Some("east"));
    assert_eq!(
        policy
            .endpoint_weights
            .iter()
            .map(|e| e.weight)
            .sum::<u16>(),
        DNS_WEIGHT_SCALE
    );
    assert_eq!(
        policy.endpoint_weights[2].disposition,
        DnsDisposition::Unusable(UnusableReason::StaleHealthCheck)
    );
    assert_eq!(
        policy.endpoint_weights[3].disposition,
        DnsDisposition::Unusable(UnusableReason::UnknownHealth)
    );
    let no_target = plan_dns_steering(request, &[west_a, west_b, forbidden]).unwrap();
    assert_eq!(no_target.selected_region, None);
    assert!(no_target.endpoint_weights.iter().all(|e| e.weight == 0));
}

#[test]
fn dns_weight_resolution_cannot_report_a_phantom_secondary_share() {
    let request = DnsRequest {
        approved_regions: &["west"],
        staleness: NOW,
    };
    let policy = plan_dns_steering(
        request,
        &[endpoint("a", "west", u64::MAX), endpoint("b", "west", 1)],
    )
    .unwrap();
    assert_eq!(policy.endpoint_weights[0].weight, DNS_WEIGHT_SCALE);
    assert_eq!(policy.endpoint_weights[1].weight, 0);
    assert_eq!(
        policy.endpoint_weights[1].disposition,
        DnsDisposition::BelowWeightResolution
    );
}

#[test]
fn keeps_existing_traffic_reserve_and_pool_boundaries_intact() {
    let reserved = Backend {
        capacity: CapacityEnvelope {
            configured_limit: 100,
            committed: 30,
            reserve: 30,
        },
        ..backend("a", "web", 0)
    };
    let exhausted = Backend {
        capacity: CapacityEnvelope {
            configured_limit: 5,
            committed: u64::MAX,
            reserve: u64::MAX,
        },
        ..backend("c", "web", 0)
    };
    let disabled = Backend {
        admitted: false,
        ..backend("d", "web", u64::MAX)
    };
    let missing = Backend {
        observation: HealthSnapshot {
            checked_at_seconds: None,
            ..GOOD
        },
        ..backend("e", "web", u64::MAX)
    };
    let cause_of_outage = Backend {
        observation: HealthSnapshot {
            status: Health::Unhealthy,
            checked_at_seconds: Some(95),
        },
        ..backend("f", "web", u64::MAX)
    };
    let input = [
        reserved,
        backend("b", "web", 20),
        exhausted,
        disabled,
        missing,
        cause_of_outage,
        backend("g", "worker", u64::MAX),
    ];
    let allocation = plan_backend_allocation(placement(90), &input).unwrap();
    assert_eq!(
        (
            allocation.demand,
            allocation.served,
            allocation.unserved,
            allocation.available
        ),
        (90, 60, 30, 60)
    );
    assert_eq!(
        allocation
            .backend_shares
            .iter()
            .map(|s| (s.backend_id, s.available, s.proposed, s.disposition))
            .collect::<Vec<_>>(),
        [
            ("a", 40, 40, BackendDisposition::LoadProposed),
            ("b", 20, 20, BackendDisposition::LoadProposed),
            (
                "c",
                0,
                0,
                BackendDisposition::Unusable(UnusableReason::NoHeadroom)
            ),
            (
                "d",
                0,
                0,
                BackendDisposition::Unusable(UnusableReason::NotAdmitted)
            ),
            (
                "e",
                0,
                0,
                BackendDisposition::Unusable(UnusableReason::MissingHealthCheck)
            ),
            (
                "f",
                0,
                0,
                BackendDisposition::Unusable(UnusableReason::ReportedUnhealthy)
            ),
            ("g", 0, 0, BackendDisposition::OutsidePool),
        ]
    );
    let empty = plan_backend_allocation(placement(10), &[]).unwrap();
    assert_eq!((empty.served, empty.unserved, empty.available), (0, 10, 0));
    assert!(empty.backend_shares.is_empty());
}

#[test]
fn huge_capacity_and_demand_keep_exact_total_and_tie_break() {
    let input = [backend("b", "web", u64::MAX), backend("a", "web", u64::MAX)];
    let allocation = plan_backend_allocation(placement(u64::MAX), &input).unwrap();
    assert_eq!(allocation.available, u128::from(u64::MAX) * 2);
    assert_eq!(allocation.served, u64::MAX);
    assert_eq!(allocation.unserved, 0);
    assert_eq!(allocation.backend_shares[0].backend_id, "a");
    assert_eq!(allocation.backend_shares[0].proposed, u64::MAX / 2 + 1);
    assert_eq!(allocation.backend_shares[1].proposed, u64::MAX / 2);
    let idle = plan_backend_allocation(placement(0), &input).unwrap();
    assert_eq!(idle.unserved, 0);
    assert!(
        idle.backend_shares
            .iter()
            .all(|row| row.proposed == 0 && row.disposition == BackendDisposition::NoLoadProposed)
    );
}

#[test]
fn backend_allocation_matches_independent_small_integer_oracle() {
    // Enumerate all plans for each bounded demand/capacity matrix. For equal
    // throughput, sum integer absolute differences from proportional shares.
    for a in 0..=4 {
        for b in 0..=4 {
            for c in 0..=4 {
                let limits = [a, b, c];
                let total = limits.into_iter().sum::<u64>();
                for demand in 0..=15 {
                    let score = |row: [u64; 3]| {
                        let served = row.into_iter().sum::<u64>();
                        let error = row
                            .into_iter()
                            .zip(limits)
                            .map(|(value, cap)| (value * total).abs_diff(cap * served))
                            .sum::<u64>();
                        (served, std::cmp::Reverse(error))
                    };
                    let mut best = score([0, 0, 0]);
                    for x in 0..=a {
                        for y in 0..=b {
                            for z in 0..=c {
                                if x + y + z <= demand {
                                    best = best.max(score([x, y, z]));
                                }
                            }
                        }
                    }
                    let input = [
                        backend("a", "web", a),
                        backend("b", "web", b),
                        backend("c", "web", c),
                    ];
                    let plan = plan_backend_allocation(placement(demand), &input).unwrap();
                    let result = std::array::from_fn(|i| plan.backend_shares[i].proposed);
                    assert_eq!(score(result), best, "limits={limits:?} demand={demand}");
                    assert_eq!(plan.served + plan.unserved, demand);
                    assert!(
                        result
                            .into_iter()
                            .zip(limits)
                            .all(|(value, cap)| value <= cap)
                    );
                }
            }
        }
    }
}

#[test]
fn rejects_duplicate_bad_or_future_snapshot_fields_with_indexed_errors() {
    let steering = |regions| DnsRequest {
        approved_regions: regions,
        staleness: NOW,
    };
    let future = DnsEndpoint {
        observation: HealthSnapshot {
            checked_at_seconds: Some(101),
            ..GOOD
        },
        ..endpoint("a", "west", 1)
    };
    let bad_ids = [endpoint("sensitive/name", "west", 1)];
    assert_eq!(
        plan_dns_steering(steering(&[]), &[]).unwrap_err().code(),
        "TRF-006"
    );
    assert_eq!(
        plan_dns_steering(steering(&["west", "west"]), &[])
            .unwrap_err()
            .code(),
        "TRF-004"
    );
    assert_eq!(
        plan_dns_steering(steering(&["west"]), &[future])
            .unwrap_err()
            .code(),
        "TRF-005"
    );
    assert_eq!(
        plan_dns_steering(steering(&["west"]), &bad_ids)
            .unwrap_err()
            .code(),
        "TRF-002"
    );
    let dup = [backend("dup", "web", 10), backend("dup", "other", 20)];
    assert_eq!(
        plan_backend_allocation(placement(1), &dup).unwrap_err(),
        TrafficError::DuplicateEndpoint
    );
    let too_many = [backend("sample", "web", 1); 257];
    assert_eq!(
        plan_backend_allocation(placement(1), &too_many)
            .unwrap_err()
            .code(),
        "TRF-001"
    );
}
