use orchestrata_solver::routing::{
    Courier, CourierMode, CourierRoute, LocationId, RouteVerificationError as Error, RouteVisit,
    RoutingProblem, Shipment, Stop, TimeWindow, VisitKind, verify_routes,
};

fn window(start: u64, end: u64) -> TimeWindow {
    TimeWindow { start, end }
}

fn stop(location: u32, start: u64, end: u64, service_time: u64) -> Stop {
    Stop {
        location: LocationId(location),
        window: window(start, end),
        service_time,
    }
}

fn visit(shipment: usize, kind: VisitKind, arrival: u64) -> RouteVisit {
    RouteVisit {
        shipment,
        kind,
        arrival,
    }
}

fn fixture() -> (RoutingProblem, CourierRoute) {
    let problem = RoutingProblem {
        couriers: vec![
            Courier {
                mode: CourierMode::Vehicle,
                depot: LocationId(0),
                shift: window(10, 27),
                capacity: 9,
            },
            Courier {
                mode: CourierMode::Bicycle,
                depot: LocationId(0),
                shift: window(10, 40),
                capacity: 9,
            },
            Courier {
                mode: CourierMode::Walking,
                depot: LocationId(0),
                shift: window(10, 40),
                capacity: 9,
            },
        ],
        shipments: vec![
            Shipment {
                pickup: stop(1, 14, 16, 2),
                delivery: stop(3, 21, 22, 1),
                weight: 4,
                allowed_modes: vec![CourierMode::Vehicle, CourierMode::Bicycle],
            },
            Shipment {
                pickup: stop(2, 18, 19, 1),
                delivery: stop(4, 24, 25, 1),
                weight: 5,
                allowed_modes: vec![CourierMode::Vehicle],
            },
        ],
    };
    let route = CourierRoute {
        courier: 0,
        departure: 10,
        visits: vec![
            visit(0, VisitKind::Pickup, 12),
            visit(1, VisitKind::Pickup, 18),
            visit(0, VisitKind::Delivery, 21),
            visit(1, VisitKind::Delivery, 24),
        ],
        depot_return: 27,
    };
    (problem, route)
}

fn mode_travel(mode: CourierMode, _: LocationId, _: LocationId, _: u64) -> Option<u64> {
    Some(match mode {
        CourierMode::Vehicle => 2,
        CourierMode::Bicycle => 3,
        CourierMode::Walking => 5,
    })
}

#[test]
fn waits_for_window_and_validates_inclusive_service_shift_and_capacity() {
    let (problem, route) = fixture();
    let mut departures = Vec::new();
    assert_eq!(
        verify_routes(&problem, &[route], |mode, from, to, depart| {
            departures.push((mode, from.0, to.0, depart));
            mode_travel(mode, from, to, depart)
        }),
        Ok(())
    );
    assert_eq!(
        departures,
        [
            (CourierMode::Vehicle, 0, 1, 10),
            (CourierMode::Vehicle, 1, 2, 16),
            (CourierMode::Vehicle, 2, 3, 19),
            (CourierMode::Vehicle, 3, 4, 22),
            (CourierMode::Vehicle, 4, 0, 25),
        ]
    );
}

#[test]
fn rejects_early_arrivals_late_service_and_inconsistent_depot_return() {
    let (problem, route) = fixture();

    let mut early_arrival = route.clone();
    early_arrival.visits[1].arrival = 17;
    assert_eq!(
        verify_routes(&problem, &[early_arrival], mode_travel),
        Err(Error::ArrivalTooEarly(1))
    );

    let mut late_service = route.clone();
    late_service.visits[1].arrival = 19;
    assert_eq!(
        verify_routes(&problem, &[late_service], mode_travel),
        Err(Error::StopWindowViolation(1))
    );

    let mut early_return = route.clone();
    early_return.depot_return = 26;
    assert_eq!(
        verify_routes(&problem, &[early_return], mode_travel),
        Err(Error::DepotReturnTooEarly(0))
    );

    let mut late_return = route;
    late_return.depot_return = 28;
    assert_eq!(
        verify_routes(&problem, &[late_return], mode_travel),
        Err(Error::ShiftViolation(0))
    );
}

#[test]
fn uses_mode_specific_travel_and_rejects_unknown_legs() {
    let (mut problem, route) = fixture();
    assert_eq!(
        verify_routes(
            &problem,
            std::slice::from_ref(&route),
            |mode, from, to, depart| {
                if from == LocationId(4) && to == LocationId(0) {
                    None
                } else {
                    mode_travel(mode, from, to, depart)
                }
            }
        ),
        Err(Error::UnreachableLeg {
            courier: 0,
            from: LocationId(4),
            to: LocationId(0)
        }),
    );

    let mut cycle = route.clone();
    cycle.courier = 1;
    assert_eq!(
        verify_routes(&problem, std::slice::from_ref(&cycle), mode_travel),
        Err(Error::ArrivalTooEarly(0))
    );
    cycle.visits[0].arrival = 13;
    assert_eq!(
        verify_routes(&problem, &[cycle], mode_travel),
        Err(Error::ForbiddenMode(1))
    );

    let mut walk = route;
    walk.courier = 2;
    problem.shipments[0]
        .allowed_modes
        .push(CourierMode::Walking);
    assert_eq!(
        verify_routes(&problem, &[walk], mode_travel),
        Err(Error::ArrivalTooEarly(0))
    );
}

#[test]
fn weighs_all_onboard_shipments_and_rejects_integer_overflow() {
    let (mut problem, route) = fixture();
    problem.couriers[0].capacity = 8;
    assert_eq!(
        verify_routes(&problem, std::slice::from_ref(&route), mode_travel),
        Err(Error::CapacityExceeded(1))
    );

    problem.couriers[0].capacity = u64::MAX;
    problem.shipments[0].weight = u64::MAX;
    assert_eq!(
        verify_routes(&problem, std::slice::from_ref(&route), mode_travel),
        Err(Error::CapacityExceeded(1))
    );

    let mut overflow_time = route;
    overflow_time.departure = u64::MAX;
    overflow_time.depot_return = u64::MAX;
    problem.couriers[0].shift = window(u64::MAX, u64::MAX);
    assert_eq!(
        verify_routes(&problem, &[overflow_time], mode_travel),
        Err(Error::ArithmeticOverflow)
    );
}

#[test]
fn rejects_duplicate_missing_reversed_and_cross_courier_stops() {
    let (problem, route) = fixture();

    let mut repeated_pickup = route.clone();
    repeated_pickup.visits.insert(1, route.visits[0]);
    assert_eq!(
        verify_routes(&problem, &[repeated_pickup], mode_travel),
        Err(Error::DuplicatePickup(0))
    );

    let mut repeated_delivery = route.clone();
    repeated_delivery.visits.push(route.visits[3]);
    assert_eq!(
        verify_routes(&problem, &[repeated_delivery], mode_travel),
        Err(Error::DuplicateDelivery(1))
    );

    let mut reversed = route.clone();
    reversed.visits.remove(0);
    assert_eq!(
        verify_routes(&problem, &[reversed], mode_travel),
        Err(Error::DeliveryBeforePickup(0))
    );

    assert_eq!(
        verify_routes(&problem, &[], mode_travel),
        Err(Error::MissingPickup(0))
    );

    let mut partial = route.clone();
    partial.visits.remove(2);
    assert_eq!(
        verify_routes(&problem, std::slice::from_ref(&partial), mode_travel),
        Err(Error::MissingDelivery(0))
    );

    let delivery = CourierRoute {
        courier: 1,
        departure: 10,
        visits: vec![route.visits[2]],
        depot_return: 27,
    };
    assert_eq!(
        verify_routes(&problem, &[partial, delivery], mode_travel),
        Err(Error::WrongDeliveryCourier(0))
    );
}

#[test]
fn rejects_duplicate_routes_unknown_indexes_and_invalid_windows() {
    let (mut problem, route) = fixture();
    assert_eq!(
        verify_routes(&problem, &[route.clone(), route.clone()], mode_travel),
        Err(Error::DuplicateCourierRoute(0)),
    );

    let mut wrong_courier = route.clone();
    wrong_courier.courier = 3;
    assert_eq!(
        verify_routes(&problem, &[wrong_courier], mode_travel),
        Err(Error::UnknownCourier(3))
    );

    let mut wrong_shipment = route.clone();
    wrong_shipment.visits[0].shipment = 2;
    assert_eq!(
        verify_routes(&problem, &[wrong_shipment], mode_travel),
        Err(Error::UnknownShipment(2))
    );

    problem.shipments[0].pickup.window = window(15, 14);
    assert_eq!(
        verify_routes(&problem, &[route], mode_travel),
        Err(Error::InvalidShipment(0))
    );
}

#[test]
fn requires_explicit_pickup_even_at_depot() {
    let (mut problem, mut route) = fixture();
    problem.shipments.truncate(1);
    problem.shipments[0].pickup.location = LocationId(0);
    route.visits = vec![visit(0, VisitKind::Delivery, 21)];
    assert_eq!(
        verify_routes(&problem, &[route], mode_travel),
        Err(Error::DeliveryBeforePickup(0))
    );
}

#[test]
fn verifies_bicycle_and_walking_routes_together_with_an_idle_vehicle() {
    let (mut problem, _) = fixture();
    problem.shipments[0].allowed_modes = vec![CourierMode::Bicycle];
    problem.shipments[1].allowed_modes = vec![CourierMode::Walking];
    let bicycle = CourierRoute {
        courier: 1,
        departure: 10,
        visits: vec![
            visit(0, VisitKind::Pickup, 13),
            visit(0, VisitKind::Delivery, 19),
        ],
        depot_return: 25,
    };
    let walking = CourierRoute {
        courier: 2,
        departure: 10,
        visits: vec![
            visit(1, VisitKind::Pickup, 15),
            visit(1, VisitKind::Delivery, 24),
        ],
        depot_return: 30,
    };
    assert_eq!(
        verify_routes(&problem, &[bicycle, walking], mode_travel),
        Ok(())
    );
}
