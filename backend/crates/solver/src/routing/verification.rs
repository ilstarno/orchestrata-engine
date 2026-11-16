use super::{
    CourierMode, CourierRoute, LocationId, RouteVerificationError as Error, RoutingProblem,
    route_cursor::RouteCursor, shipment_status::ShipmentStatus,
};

/// Verifies a complete, depot-to-depot pickup and delivery plan.
///
/// All couriers start empty. Preloaded freight, transfers, and split shipments
/// are not supported. Idle couriers may be omitted. All shipments must appear
/// with exactly one pickup and a later delivery visit on the same route.
/// The trusted travel source returns a leg duration or `None` for an unknown
/// or unreachable leg. It is sampled at the preceding service finish or depot
/// departure; there is no movement between stops at the same location.
/// Arrivals may include slack, and waiting until a stop opens is permitted.
pub fn verify_routes(
    problem: &RoutingProblem,
    routes: &[CourierRoute],
    mut travel: impl FnMut(CourierMode, LocationId, LocationId, u64) -> Option<u64>,
) -> Result<(), Error> {
    validate_problem(problem)?;

    let mut used = vec![false; problem.couriers.len()];
    let mut statuses = vec![ShipmentStatus::Pending; problem.shipments.len()];
    for route in routes {
        let courier = problem
            .couriers
            .get(route.courier)
            .ok_or(Error::UnknownCourier(route.courier))?;
        if std::mem::replace(&mut used[route.courier], true) {
            return Err(Error::DuplicateCourierRoute(route.courier));
        }
        if !courier.shift.contains(route.departure) || !courier.shift.contains(route.depot_return) {
            return Err(Error::ShiftViolation(route.courier));
        }

        let mut cursor = RouteCursor {
            location: courier.depot,
            departure: route.departure,
            load: 0,
        };
        for visit in &route.visits {
            let shipment = problem
                .shipments
                .get(visit.shipment)
                .ok_or(Error::UnknownShipment(visit.shipment))?;
            cursor.visit(
                route.courier,
                courier,
                visit,
                shipment,
                &mut statuses[visit.shipment],
                &mut travel,
            )?;
        }

        let earliest_return =
            cursor.earliest_arrival(route.courier, courier.mode, courier.depot, &mut travel)?;
        if route.depot_return < earliest_return {
            return Err(Error::DepotReturnTooEarly(route.courier));
        }
    }

    for (shipment, status) in statuses.into_iter().enumerate() {
        match status {
            ShipmentStatus::Pending => return Err(Error::MissingPickup(shipment)),
            ShipmentStatus::Onboard(_) => return Err(Error::MissingDelivery(shipment)),
            ShipmentStatus::Delivered => {}
        }
    }

    Ok(())
}

fn validate_problem(problem: &RoutingProblem) -> Result<(), Error> {
    for (index, courier) in problem.couriers.iter().enumerate() {
        if !courier.shift.is_valid() {
            return Err(Error::InvalidShift(index));
        }
    }

    for (index, shipment) in problem.shipments.iter().enumerate() {
        if !shipment.pickup.window.is_valid()
            || !shipment.delivery.window.is_valid()
            || shipment.allowed_modes.is_empty()
        {
            return Err(Error::InvalidShipment(index));
        }
    }

    Ok(())
}
