use super::{CourierMode, CourierRoute, LocationId, RouteVisit, RoutingProblem, VisitKind};

pub(super) fn simulate(
    problem: &RoutingProblem,
    courier_index: usize,
    mut visits: Vec<RouteVisit>,
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Option<CourierRoute> {
    let courier = &problem.couriers[courier_index];
    let departure = courier.shift.start;
    let mut time = departure;
    let mut previous_location = courier.depot;
    let mut load = 0_u64;

    for visit in &mut visits {
        let shipment = &problem.shipments[visit.shipment];
        let stop = match visit.kind {
            VisitKind::Pickup => shipment.pickup,
            VisitKind::Delivery => shipment.delivery,
        };
        let duration = leg_time(courier.mode, previous_location, stop.location, travel)?;
        let arrival = time.checked_add(duration)?;
        let finish = arrival
            .max(stop.window.start)
            .checked_add(stop.service_time)?;
        if finish > stop.window.end || finish > courier.shift.end {
            return None;
        }
        load = match visit.kind {
            VisitKind::Pickup => load.checked_add(shipment.weight)?,
            VisitKind::Delivery => load.checked_sub(shipment.weight)?,
        };
        if load > courier.capacity {
            return None;
        }

        visit.arrival = arrival;
        time = finish;
        previous_location = stop.location;
    }

    let returning = leg_time(courier.mode, previous_location, courier.depot, travel)?;
    let depot_return = time.checked_add(returning)?;
    (depot_return <= courier.shift.end).then_some(CourierRoute {
        courier: courier_index,
        departure,
        visits,
        depot_return,
    })
}

fn leg_time(
    mode: CourierMode,
    from: LocationId,
    to: LocationId,
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Option<u64> {
    if from == to {
        Some(0)
    } else {
        travel(mode, from, to)
    }
}
