use super::{
    CourierMode, CourierRoute, LocationId, RouteConstruction, RouteConstructionError as Error,
    RouteConstructionStop, RouteVerificationError, RouteVisit, RoutingProblem, VisitKind,
    construction_limits::{
        DISTINCT_COURIER_MODES, MAX_CONSTRUCTION_COURIERS, MAX_CONSTRUCTION_INSERTIONS,
        MAX_CONSTRUCTION_SHIPMENTS,
    },
    construction_simulation::simulate,
    verification::validate_problem,
    verify_routes,
};

/// Attempts deterministic pickup/delivery insertion on a strictly bounded instance.
///
/// The caller supplies a repeatable, static, mode-aware travel lookup in the
/// problem's clock units. `None` means unknown or unreachable; no lookup is
/// needed for same-location legs. Each route starts at the courier's shift
/// start and its stops use earliest arrival with waiting as needed. Shipments
/// are considered in input order; paired visits are inserted in the position
/// with the least added time until depot return. Ties use original courier,
/// pickup-position, then delivery-position order. Existing routes keep their
/// relative visit order. Any valid insertion already found is retained if the
/// budget expires mid-shipment. Invalid inputs are errors, while skipped or
/// unsuccessfully inserted shipments remain unassigned. This is not an
/// infeasibility proof or an optimality guarantee, even after a complete pass.
pub fn construct_routes(
    problem: &RoutingProblem,
    max_insertions: u32,
    travel: impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Result<RouteConstruction, Error> {
    validate_construction(problem, max_insertions)?;

    let mut remaining = max_insertions;
    let mut routes = vec![None; problem.couriers.len()];
    let mut assigned = vec![false; problem.shipments.len()];
    let mut stop = RouteConstructionStop::CompletedPass;
    for (shipment, placed) in assigned.iter_mut().enumerate() {
        let (route, stopped) = best_insertion(problem, &routes, shipment, &mut remaining, &travel);
        if let Some(route) = route {
            let courier = route.courier;
            routes[courier] = Some(route);
            *placed = true;
        }
        if stopped {
            stop = RouteConstructionStop::InsertionLimit;
            break;
        }
    }

    let routes: Vec<_> = routes.into_iter().flatten().collect();
    let unassigned: Vec<_> = assigned
        .iter()
        .enumerate()
        .filter_map(|(index, &placed)| (!placed).then_some(index))
        .collect();
    verify_construction(problem, &routes, &assigned, &travel)?;

    Ok(RouteConstruction {
        routes,
        unassigned,
        evaluated: max_insertions - remaining,
        stop,
    })
}

fn validate_construction(problem: &RoutingProblem, max_insertions: u32) -> Result<(), Error> {
    if problem.couriers.len() > MAX_CONSTRUCTION_COURIERS {
        return Err(Error::TooManyCouriers(problem.couriers.len()));
    }
    if problem.shipments.len() > MAX_CONSTRUCTION_SHIPMENTS {
        return Err(Error::TooManyShipments(problem.shipments.len()));
    }
    if max_insertions > MAX_CONSTRUCTION_INSERTIONS {
        return Err(Error::InsertionLimitTooHigh(max_insertions));
    }
    for (index, shipment) in problem.shipments.iter().enumerate() {
        let modes = &shipment.allowed_modes;
        if modes.len() > DISTINCT_COURIER_MODES
            || modes
                .iter()
                .enumerate()
                .any(|(position, mode)| modes[..position].contains(mode))
        {
            return Err(Error::InvalidProblem(
                RouteVerificationError::InvalidShipment(index),
            ));
        }
    }
    validate_problem(problem).map_err(Error::InvalidProblem)
}

fn best_insertion(
    problem: &RoutingProblem,
    routes: &[Option<CourierRoute>],
    shipment_index: usize,
    remaining: &mut u32,
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> (Option<CourierRoute>, bool) {
    let shipment = &problem.shipments[shipment_index];
    let mut best: Option<(i128, CourierRoute)> = None;

    for (courier_index, current) in routes.iter().enumerate() {
        let courier = &problem.couriers[courier_index];
        if shipment.weight > courier.capacity || !shipment.allowed_modes.contains(&courier.mode) {
            continue;
        }
        let (existing, prior_return) = current
            .as_ref()
            .map_or((&[][..], courier.shift.start), |route| {
                (route.visits.as_slice(), route.depot_return)
            });
        for pickup_position in 0..=existing.len() {
            for delivery_position in pickup_position + 1..=existing.len() + 1 {
                if *remaining == 0 {
                    return (best.map(|(_, route)| route), true);
                }
                *remaining -= 1;

                let mut visits = existing.to_vec();
                visits.insert(
                    pickup_position,
                    open_visit(shipment_index, VisitKind::Pickup),
                );
                visits.insert(
                    delivery_position,
                    open_visit(shipment_index, VisitKind::Delivery),
                );
                let Some(candidate) = simulate(problem, courier_index, visits, travel) else {
                    continue;
                };
                let added_time = i128::from(candidate.depot_return) - i128::from(prior_return);
                if best.as_ref().is_none_or(|(prior, _)| added_time < *prior) {
                    best = Some((added_time, candidate));
                }
            }
        }
    }

    (best.map(|(_, route)| route), false)
}

const fn open_visit(shipment: usize, kind: VisitKind) -> RouteVisit {
    RouteVisit {
        shipment,
        kind,
        arrival: 0,
    }
}

fn verify_construction(
    problem: &RoutingProblem,
    routes: &[CourierRoute],
    assigned: &[bool],
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Result<(), Error> {
    if assigned.iter().all(|&placed| placed) {
        return verify_routes(problem, routes, |mode, from, to, _| travel(mode, from, to))
            .map_err(Error::VerificationFailed);
    }

    // Reindex only for the completeness checker; returned routes keep original IDs.
    let mut projected_ids = vec![None; assigned.len()];
    let mut shipments = Vec::new();
    for (index, &placed) in assigned.iter().enumerate() {
        if placed {
            projected_ids[index] = Some(shipments.len());
            shipments.push(problem.shipments[index].clone());
        }
    }
    let mut projected_routes = routes.to_vec();
    for route in &mut projected_routes {
        for visit in &mut route.visits {
            visit.shipment = projected_ids.get(visit.shipment).copied().flatten().ok_or(
                Error::VerificationFailed(RouteVerificationError::UnknownShipment(visit.shipment)),
            )?;
        }
    }
    let projection = RoutingProblem {
        couriers: problem.couriers.clone(),
        shipments,
    };
    verify_routes(&projection, &projected_routes, |mode, from, to, _| {
        travel(mode, from, to)
    })
    .map_err(Error::VerificationFailed)
}
