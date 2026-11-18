use orchestrata_solver::routing::{coverage::*, *};
use std::collections::HashMap;
type Travel = HashMap<(CourierMode, LocationId, LocationId), u64>;
fn stop(location: u32, end: u64, service_time: u64) -> Stop {
    Stop {
        location: LocationId(location),
        window: TimeWindow { start: 0, end },
        service_time,
    }
}
fn courier(mode: CourierMode, end: u64, capacity: u64) -> Courier {
    Courier {
        mode,
        depot: LocationId(0),
        shift: TimeWindow { start: 0, end },
        capacity,
    }
}
fn bound(p: &RoutingProblem, t: &Travel) -> CoverageCertificate {
    let incumbent = construct_routes(p, 20_000, |m, f, z| t.get(&(m, f, z)).copied()).unwrap();
    let CoverageOutcome::Complete(c) =
        certify_coverage(p, &incumbent.routes, MAX_COVERAGE_WORK, |m, f, z| {
            t.get(&(m, f, z)).copied()
        })
        .unwrap()
    else {
        panic!("complete bound required")
    };
    assert!(verify_flow_witness(&c.service));
    assert!(verify_flow_witness(&c.conflicts));
    *c
}
// Independent exhaustive traversal: original arcs, all legal pickup/delivery
// sequences and every disjoint combination of feasible courier subsets.
fn exact(p: &RoutingProblem, t: &Travel) -> usize {
    fn walk(
        p: &RoutingProblem,
        t: &Travel,
        c: &Courier,
        cursor: (LocationId, u64, u64),
        state: &mut [u8],
        masks: &mut Vec<usize>,
    ) {
        let (loc, time, load) = cursor;
        if !state.contains(&1) {
            let back = if loc == c.depot {
                Some(0)
            } else {
                t.get(&(c.mode, loc, c.depot)).copied()
            };
            if back.is_some_and(|d| time + d <= c.shift.end) {
                masks.push(
                    state
                        .iter()
                        .enumerate()
                        .filter(|(_, s)| **s == 2)
                        .map(|(j, _)| 1 << j)
                        .sum(),
                );
            }
        }
        for j in 0..state.len() {
            if state[j] == 2 {
                continue;
            }
            let s = &p.shipments[j];
            if !s.allowed_modes.contains(&c.mode) {
                continue;
            }
            let pickup = state[j] == 0;
            let next_load = if pickup {
                load + s.weight
            } else {
                load - s.weight
            };
            if next_load > c.capacity {
                continue;
            }
            let stop = if pickup { &s.pickup } else { &s.delivery };
            let d = if loc == stop.location {
                Some(0)
            } else {
                t.get(&(c.mode, loc, stop.location)).copied()
            };
            let Some(d) = d else { continue };
            let finish = (time + d).max(stop.window.start) + stop.service_time;
            if finish > stop.window.end || finish > c.shift.end {
                continue;
            }
            state[j] += 1;
            walk(p, t, c, (stop.location, finish, next_load), state, masks);
            state[j] -= 1;
        }
    }
    let mut covered = vec![0usize];
    for c in &p.couriers {
        let mut masks = Vec::new();
        walk(
            p,
            t,
            c,
            (c.depot, c.shift.start, 0),
            &mut vec![0; p.shipments.len()],
            &mut masks,
        );
        masks.sort_unstable();
        masks.dedup();
        let mut next = Vec::new();
        for &a in &covered {
            for &b in &masks {
                if a & b == 0 {
                    next.push(a | b);
                }
            }
        }
        next.sort_unstable();
        next.dedup();
        covered = next;
    }
    covered
        .iter()
        .map(|m| m.count_ones() as usize)
        .max()
        .unwrap_or(0)
}
#[test]
fn sparse_nonmetric_paths_must_not_exclude_serviceable_jobs() {
    let mode = CourierMode::Vehicle;
    let p = RoutingProblem {
        couriers: vec![courier(mode, 10, 1)],
        shipments: vec![
            Shipment {
                pickup: stop(1, 10, 0),
                delivery: stop(1, 10, 0),
                weight: 1,
                allowed_modes: vec![mode],
            },
            Shipment {
                pickup: stop(2, 10, 0),
                delivery: stop(2, 10, 0),
                weight: 1,
                allowed_modes: vec![mode],
            },
        ],
    };
    let t = HashMap::from([
        ((mode, LocationId(0), LocationId(2)), 1),
        ((mode, LocationId(2), LocationId(1)), 1),
        ((mode, LocationId(1), LocationId(0)), 1),
    ]);
    assert_eq!(exact(&p, &t), 2);
    let c = bound(&p, &t);
    assert_eq!(c.upper_bound, 2);
    assert_eq!(c.service.eligible_slots, vec![vec![0], vec![0]]);
}
#[test]
fn exact_small_nonmetric_directed_mixed_mode_cases() {
    let modes = [
        CourierMode::Vehicle,
        CourierMode::Bicycle,
        CourierMode::Walking,
    ];
    let mut seed = 47u64;
    let mut next = |n| {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (seed >> 32) % n
    };
    let mut tight = 0;
    for case in 0..512 {
        let count = if case % 16 == 0 { 4 } else { 3 };
        let mut p = RoutingProblem {
            couriers: vec![],
            shipments: vec![],
        };
        for i in 0..2 {
            p.couriers
                .push(courier(modes[(case + i) % 3], 12 + next(35), 1 + next(5)));
        }
        for j in 0..count {
            let mut pickup = stop(1 + 2 * j as u32, 8 + next(30), next(3));
            pickup.window.start = next(6);
            let mut delivery = stop(2 + 2 * j as u32, 10 + next(35), next(3));
            delivery.window.start = next(8);
            p.shipments.push(Shipment {
                pickup,
                delivery,
                weight: 1 + next(3),
                allowed_modes: if next(3) == 0 {
                    vec![modes[j % 3]]
                } else {
                    modes.to_vec()
                },
            });
        }
        for c in &mut p.couriers {
            c.shift.start = next(7);
            c.depot = LocationId(next((2 * count + 1) as u64) as u32);
        }
        let mut t = Travel::new();
        for m in modes {
            for a in 0..=2 * count {
                for z in 0..=2 * count {
                    if a != z && next(5) != 0 {
                        t.insert((m, LocationId(a as u32), LocationId(z as u32)), next(13));
                    }
                }
            }
        }
        let optimum = exact(&p, &t);
        let c = bound(&p, &t);
        assert!(
            c.upper_bound >= optimum,
            "case {case}: {c:?}, exact {optimum}"
        );
        assert!(c.incumbent_covered <= optimum);
        assert_eq!(c.maximum_coverage, c.incumbent_covered == c.upper_bound);
        if c.maximum_coverage {
            assert_eq!(c.incumbent_covered, optimum);
        }
        if c.upper_bound == optimum {
            tight += 1;
        }
    }
    assert!(tight > 100);
}
#[test]
fn bounded_failure_replay_and_witness_tampering() {
    let mode = CourierMode::Walking;
    let p = RoutingProblem {
        couriers: vec![courier(mode, 10, 1)],
        shipments: vec![Shipment {
            pickup: stop(0, 10, 2),
            delivery: stop(0, 10, 2),
            weight: 1,
            allowed_modes: vec![mode],
        }],
    };
    let plan = construct_routes(&p, 10, |_, _, _| None).unwrap();
    for work in [0, 1, 5] {
        assert!(
            matches!(certify_coverage(&p,&plan.routes,work,|_,_,_|None).unwrap(),CoverageOutcome::Incomplete{work:w} if w==work)
        );
    }
    assert_eq!(
        certify_coverage(&p, &[], MAX_COVERAGE_WORK + 1, |_, _, _| None),
        Err(CoverageError::LimitExceeded)
    );
    let mut bad = plan.routes;
    bad[0].visits.pop();
    assert!(matches!(
        certify_coverage(&p, &bad, MAX_COVERAGE_WORK, |_, _, _| None),
        Err(CoverageError::InvalidIncumbent(_))
    ));
    let c = bound(&p, &Travel::new());
    assert!(c.maximum_coverage);
    let mut w = c.service;
    w.cut_capacity += 1;
    assert!(!verify_flow_witness(&w));
    w.cut_capacity -= 1;
    w.matching[0] = None;
    assert!(!verify_flow_witness(&w));
}
#[test]
fn service_bound_and_extreme_arithmetic() {
    let mode = CourierMode::Vehicle;
    let p = RoutingProblem {
        couriers: vec![courier(mode, 10, 10)],
        shipments: (0..3)
            .map(|_| Shipment {
                pickup: stop(0, 10, 2),
                delivery: stop(0, 10, 2),
                weight: 1,
                allowed_modes: vec![mode],
            })
            .collect(),
    };
    let c = bound(&p, &Travel::new());
    assert_eq!(c.service.cut_capacity, 2);
    assert_eq!(c.upper_bound, 2);
    assert_eq!(exact(&p, &Travel::new()), 2);
    let huge = RoutingProblem {
        couriers: vec![courier(mode, u64::MAX, u64::MAX)],
        shipments: vec![Shipment {
            pickup: stop(1, u64::MAX, u64::MAX),
            delivery: stop(2, u64::MAX, u64::MAX),
            weight: u64::MAX,
            allowed_modes: vec![mode],
        }],
    };
    let c = bound(
        &huge,
        &HashMap::from([
            ((mode, LocationId(0), LocationId(1)), u64::MAX),
            ((mode, LocationId(1), LocationId(2)), u64::MAX),
        ]),
    );
    assert_eq!(c.upper_bound, 0);
}

#[test]
fn largest_supported_instance_and_empty_problem_are_bounded() {
    use std::cell::Cell;
    let modes = [
        CourierMode::Vehicle,
        CourierMode::Bicycle,
        CourierMode::Walking,
    ];
    let p = RoutingProblem {
        couriers: (0..16)
            .map(|i| Courier {
                mode: modes[i % 3],
                depot: LocationId(64 + i as u32),
                shift: TimeWindow {
                    start: 0,
                    end: 1000,
                },
                capacity: 32,
            })
            .collect(),
        shipments: (0..32)
            .map(|i| Shipment {
                pickup: stop(2 * i, 1000, 1),
                delivery: stop(2 * i + 1, 1000, 1),
                weight: 1,
                allowed_modes: modes.to_vec(),
            })
            .collect(),
    };
    let calls = Cell::new(0usize);
    let lookup = |_, _, _| {
        calls.set(calls.get() + 1);
        Some(1)
    };
    assert_eq!(
        certify_coverage(&p, &[], 1000, lookup).unwrap(),
        CoverageOutcome::Incomplete { work: 1000 }
    );
    calls.set(0);
    let CoverageOutcome::Complete(c) =
        certify_coverage(&p, &[], MAX_COVERAGE_WORK, lookup).unwrap()
    else {
        panic!("bounded completion")
    };
    assert_eq!(
        (c.upper_bound, c.incumbent_covered, c.maximum_coverage),
        (32, 0, false)
    );
    assert!(calls.get() <= 3 * 80 * 79);
    assert!(c.work < MAX_COVERAGE_WORK);
    let empty = RoutingProblem {
        couriers: vec![],
        shipments: vec![],
    };
    let CoverageOutcome::Complete(c) =
        certify_coverage(&empty, &[], MAX_COVERAGE_WORK, |_, _, _| None).unwrap()
    else {
        panic!("empty completion")
    };
    assert_eq!((c.upper_bound, c.maximum_coverage), (0, true));
    let mut oversized = p;
    oversized.shipments.push(oversized.shipments[0].clone());
    assert_eq!(
        certify_coverage(&oversized, &[], MAX_COVERAGE_WORK, |_, _, _| Some(1)),
        Err(CoverageError::LimitExceeded)
    );
}

#[test]
fn every_legal_two_job_order_can_prevent_a_false_conflict() {
    for order in [
        [0, 1, 2, 3],
        [0, 2, 1, 3],
        [0, 2, 3, 1],
        [2, 3, 0, 1],
        [2, 0, 3, 1],
        [2, 0, 1, 3],
    ] {
        let mut stops = [stop(0, 10, 1); 4];
        for (i, &event) in order.iter().enumerate() {
            stops[event].window = TimeWindow {
                start: i as u64,
                end: i as u64 + 1,
            };
        }
        let mode = CourierMode::Vehicle;
        let p = RoutingProblem {
            couriers: vec![courier(mode, 4, 2)],
            shipments: vec![
                Shipment {
                    pickup: stops[0],
                    delivery: stops[1],
                    weight: 1,
                    allowed_modes: vec![mode],
                },
                Shipment {
                    pickup: stops[2],
                    delivery: stops[3],
                    weight: 1,
                    allowed_modes: vec![mode],
                },
            ],
        };
        let c = bound(&p, &Travel::new());
        assert_eq!(exact(&p, &Travel::new()), 2);
        assert_eq!(c.upper_bound, 2);
        assert_eq!(c.cliques.len(), 2);
    }
}
