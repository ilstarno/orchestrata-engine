use std::cell::Cell;

use orchestrata_solver::routing::{
    Courier, CourierMode, CourierRoute, LocationId, MAX_CONSTRUCTION_COURIERS,
    MAX_CONSTRUCTION_INSERTIONS, MAX_CONSTRUCTION_SHIPMENTS, RouteConstructionError,
    RouteConstructionStop, RouteVerificationError, RoutingProblem, Shipment, Stop, TimeWindow,
    VisitKind, construct_routes, verify_routes,
};

fn courier(mode: CourierMode, capacity: u64, end: u64) -> Courier {
    Courier {
        mode,
        depot: LocationId(0),
        shift: TimeWindow { start: 0, end },
        capacity,
    }
}

fn stop(location: u32, start: u64, end: u64, service_time: u64) -> Stop {
    Stop {
        location: LocationId(location),
        window: TimeWindow { start, end },
        service_time,
    }
}

fn shipment(pickup: Stop, delivery: Stop, weight: u64, allowed_modes: &[CourierMode]) -> Shipment {
    Shipment {
        pickup,
        delivery,
        weight,
        allowed_modes: allowed_modes.to_vec(),
    }
}

fn line_travel(mode: CourierMode, from: LocationId, to: LocationId) -> Option<u64> {
    let scale = match mode {
        CourierMode::Vehicle => 1,
        CourierMode::Bicycle => 2,
        CourierMode::Walking => 4,
    };
    Some(u64::from(from.0.abs_diff(to.0)) * scale)
}

fn retail_transfers() -> RoutingProblem {
    RoutingProblem {
        couriers: vec![courier(CourierMode::Vehicle, 8, 30)],
        shipments: vec![
            shipment(
                stop(1, 1, 4, 1),
                stop(4, 4, 7, 1),
                5,
                &[CourierMode::Vehicle],
            ),
            shipment(
                stop(2, 3, 5, 1),
                stop(5, 0, 20, 1),
                3,
                &[CourierMode::Vehicle],
            ),
        ],
    }
}

#[test]
fn pairs_can_be_inserted_between_prior_visits_and_replayed_by_the_verifier() {
    let problem = retail_transfers();
    let result = construct_routes(&problem, 20, line_travel).unwrap();
    assert_eq!(result, construct_routes(&problem, 20, line_travel).unwrap());
    assert_eq!(result.stop, RouteConstructionStop::CompletedPass);
    assert_eq!(result.evaluated, 7);
    assert!(result.unassigned.is_empty());
    assert_eq!(result.routes.len(), 1);
    let active = &result.routes[0];
    assert_eq!(active.departure, 0);
    assert_eq!(active.depot_return, 14);
    assert_eq!(
        active
            .visits
            .iter()
            .map(|visit| (visit.shipment, visit.kind, visit.arrival))
            .collect::<Vec<_>>(),
        [
            (0, VisitKind::Pickup, 1),
            (1, VisitKind::Pickup, 3),
            (0, VisitKind::Delivery, 6),
            (1, VisitKind::Delivery, 8),
        ],
    );
    verify_routes(&problem, &result.routes, |mode, from, to, _| {
        line_travel(mode, from, to)
    })
    .unwrap();
}

#[test]
fn capacity_respects_simultaneous_load_and_waiting_ends_before_shift() {
    let problem = RoutingProblem {
        couriers: vec![courier(CourierMode::Vehicle, 5, 15)],
        shipments: vec![
            shipment(
                stop(0, 2, 3, 1),
                stop(2, 8, 9, 1),
                5,
                &[CourierMode::Vehicle],
            ),
            shipment(
                stop(1, 1, 7, 1),
                stop(3, 10, 13, 1),
                1,
                &[CourierMode::Vehicle],
            ),
        ],
    };

    let result = construct_routes(&problem, 20, line_travel).unwrap();
    assert_eq!(result.unassigned, [1]);
    assert_eq!(result.routes[0].visits[0].arrival, 0); // waits until time 2 to start service
    assert_eq!(result.routes[0].visits[1].arrival, 5); // waits until time 8
    assert_eq!(result.routes[0].depot_return, 11);
    assert_eq!(result.stop, RouteConstructionStop::CompletedPass);
}

#[test]
fn considers_mode_specific_legs_and_preserves_original_indices_after_a_skip() {
    let mut problem = retail_transfers();
    problem.couriers.push(courier(CourierMode::Bicycle, 2, 40));
    problem.shipments.insert(
        1,
        shipment(
            stop(0, 0, 5, 1),
            stop(2, 1, 12, 1),
            7,
            &[CourierMode::Bicycle],
        ),
    );
    problem.shipments.push(shipment(
        stop(0, 0, 5, 1),
        stop(9, 1, 25, 1),
        1,
        &[CourierMode::Vehicle, CourierMode::Bicycle],
    ));

    let travel = |mode, from: LocationId, to: LocationId| {
        assert_ne!(from, to);
        if mode == CourierMode::Vehicle && (from.0 == 9 || to.0 == 9) {
            return None;
        }
        line_travel(mode, from, to)
    };
    let result = construct_routes(&problem, 40, travel).unwrap();
    assert_eq!(result.unassigned, [1]);
    assert_eq!(
        result
            .routes
            .iter()
            .map(|route| route.courier)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(
        result.routes[0]
            .visits
            .iter()
            .map(|visit| visit.shipment)
            .collect::<Vec<_>>(),
        [0, 2, 0, 2],
    );
    assert_eq!(
        result.routes[1]
            .visits
            .iter()
            .map(|visit| visit.shipment)
            .collect::<Vec<_>>(),
        [3, 3],
    );
}

#[test]
fn budget_keeps_valid_routes_and_leaves_unvisited_shipments_unknown() {
    let mut problem = retail_transfers();
    problem.couriers.push(problem.couriers[0]);

    let empty = construct_routes(&problem, 0, line_travel).unwrap();
    assert!(empty.routes.is_empty());
    assert_eq!(empty.unassigned, [0, 1]);
    assert_eq!(empty.evaluated, 0);
    assert_eq!(empty.stop, RouteConstructionStop::InsertionLimit);

    let partial = construct_routes(&problem, 1, line_travel).unwrap();
    assert_eq!(partial.routes.len(), 1);
    assert_eq!(partial.routes[0].courier, 0);
    assert_eq!(partial.unassigned, [1]);
    assert_eq!(partial.evaluated, 1);
    assert_eq!(partial.stop, RouteConstructionStop::InsertionLimit);

    problem.shipments.truncate(1);
    let tied = construct_routes(&problem, 2, line_travel).unwrap();
    assert!(tied.unassigned.is_empty());
    assert_eq!(tied.routes[0].courier, 0);
    assert_eq!(tied.evaluated, 2);
    assert_eq!(tied.stop, RouteConstructionStop::CompletedPass);
}

#[test]
fn lack_of_return_leg_or_clock_overflow_leaves_an_unassigned_shipment() {
    let mut problem = retail_transfers();
    problem.shipments.truncate(1);
    let missing_return = construct_routes(&problem, 10, |mode, from, to: LocationId| {
        (to.0 != 0).then(|| line_travel(mode, from, to).unwrap())
    })
    .unwrap();
    assert!(missing_return.routes.is_empty());
    assert_eq!(missing_return.unassigned, [0]);
    assert_eq!(missing_return.stop, RouteConstructionStop::CompletedPass);

    problem.couriers[0].shift.start = 1;
    let overflowing = construct_routes(&problem, 10, |_, _, _| Some(u64::MAX)).unwrap();
    assert_eq!(overflowing.unassigned, [0]);
    assert_eq!(overflowing.evaluated, 1);
}

#[test]
fn validates_caps_and_structure_before_requesting_travel() {
    let valid = retail_transfers();
    let never = |_, _, _| panic!("invalid inputs cannot start route simulation");
    let excessive_couriers = RoutingProblem {
        couriers: vec![valid.couriers[0]; MAX_CONSTRUCTION_COURIERS + 1],
        shipments: valid.shipments.clone(),
    };
    assert_eq!(
        construct_routes(&excessive_couriers, 1, never),
        Err(RouteConstructionError::TooManyCouriers(
            MAX_CONSTRUCTION_COURIERS + 1
        )),
    );
    let excessive_shipments = RoutingProblem {
        couriers: valid.couriers.clone(),
        shipments: vec![valid.shipments[0].clone(); MAX_CONSTRUCTION_SHIPMENTS + 1],
    };
    assert_eq!(
        construct_routes(&excessive_shipments, 1, never),
        Err(RouteConstructionError::TooManyShipments(
            MAX_CONSTRUCTION_SHIPMENTS + 1
        )),
    );
    assert_eq!(
        construct_routes(&valid, MAX_CONSTRUCTION_INSERTIONS + 1, never),
        Err(RouteConstructionError::InsertionLimitTooHigh(
            MAX_CONSTRUCTION_INSERTIONS + 1
        )),
    );

    let mut invalid_modes = valid.clone();
    invalid_modes.shipments[0]
        .allowed_modes
        .push(CourierMode::Vehicle);
    assert_eq!(
        construct_routes(&invalid_modes, 1, never),
        Err(RouteConstructionError::InvalidProblem(
            RouteVerificationError::InvalidShipment(0)
        )),
    );

    let mut invalid_shift = valid;
    invalid_shift.couriers[0].shift.start = 31;
    assert_eq!(
        construct_routes(&invalid_shift, 1, never),
        Err(RouteConstructionError::InvalidProblem(
            RouteVerificationError::InvalidShift(0)
        )),
    );
}

#[test]
fn independently_checks_generated_routes_even_when_a_bad_travel_lookup_changes() {
    let mut problem = retail_transfers();
    problem.shipments.truncate(1);
    let count = Cell::new(0);
    let unstable = |mode, from, to| {
        let previous = count.get();
        count.set(previous + 1);
        if previous < 3 {
            line_travel(mode, from, to)
        } else {
            Some(10)
        }
    };

    assert_eq!(
        construct_routes(&problem, 1, unstable),
        Err(RouteConstructionError::VerificationFailed(
            RouteVerificationError::ArrivalTooEarly(0)
        )),
    );
}

#[test]
fn empty_problem_completes_without_a_budget_or_any_travel() {
    let problem = RoutingProblem {
        couriers: vec![],
        shipments: vec![],
    };
    let actual = construct_routes(&problem, 0, |_, _, _| panic!("no travel")).unwrap();
    assert_eq!(actual.routes, Vec::<CourierRoute>::new());
    assert!(actual.unassigned.is_empty());
    assert_eq!(actual.evaluated, 0);
    assert_eq!(actual.stop, RouteConstructionStop::CompletedPass);
}
