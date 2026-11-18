use super::{CompiledRoute, DispatchCoverageError as Error, DispatchSolution};
use orchestrata_solver::routing::{
    CourierRoute, RouteVisit, VisitKind,
    coverage::{CoverageOutcome, certify_coverage},
};

/// Explicit, bounded coverage-only certification. Does not run during solving.
/// Checks IDs, supplied timing/load units, service fields, total elapsed time,
/// and the unassigned partition before independently replaying native visits.
/// Solver labels and insertion counters are not proof inputs.
pub fn certify_route_coverage(
    c: &CompiledRoute,
    result: &DispatchSolution,
    max_work: u32,
) -> Result<CoverageOutcome, Error> {
    let routes = replay_input(c, result)?;
    certify_coverage(&c.problem, &routes, max_work, |m, f, t| {
        c.travel.get(&(m, f, t)).copied()
    })
    .map_err(Error::Native)
}

/// Explicit stronger capacity relaxation, with one total deterministic budget.
pub fn certify_route_coverage_with_travel(
    c: &CompiledRoute,
    result: &DispatchSolution,
    max_work: u32,
) -> Result<super::TravelCoverageOutcome, Error> {
    let routes = replay_input(c, result)?;
    orchestrata_solver::routing::coverage::certify_coverage_with_travel(
        &c.problem,
        &routes,
        max_work,
        |m, f, t| c.travel.get(&(m, f, t)).copied(),
    )
    .map_err(Error::Native)
}
fn replay_input(c: &CompiledRoute, result: &DispatchSolution) -> Result<Vec<CourierRoute>, Error> {
    if result.schema != "route-dispatch-result-v1"
        || result.time_unit != "minutes"
        || result.load_unit != "kg"
        || result.routes.len() > c.problem.couriers.len()
        || result.unassigned.len() > c.problem.shipments.len()
        || result.routes.iter().any(|r| r.visits.len() > 64)
    {
        return Err(Error::InvalidResult);
    }
    let mut routes = Vec::new();
    let mut used = vec![false; c.problem.shipments.len()];
    let mut elapsed = 0u64;
    for r in &result.routes {
        let courier = c
            .snapshot
            .couriers
            .iter()
            .position(|x| x.id == r.courier_id)
            .ok_or(Error::InvalidResult)?;
        elapsed = elapsed
            .checked_add(
                r.depot_return_minute
                    .checked_sub(r.depot_departure_minute)
                    .ok_or(Error::InvalidResult)?,
            )
            .ok_or(Error::InvalidResult)?;
        let mut visits = Vec::new();
        for v in &r.visits {
            let shipment = c
                .snapshot
                .shipments
                .iter()
                .position(|x| x.id == v.shipment_id)
                .ok_or(Error::InvalidResult)?;
            let s = &c.snapshot.shipments[shipment];
            let (kind, stop) = match v.kind {
                "pickup" => (VisitKind::Pickup, &s.pickup),
                "delivery" => (VisitKind::Delivery, &s.delivery),
                _ => return Err(Error::InvalidResult),
            };
            let start = v.arrival_minute.max(stop.window.start);
            if v.location != stop.location
                || v.service_start_minute != start
                || Some(v.service_end_minute) != start.checked_add(stop.service_minutes)
            {
                return Err(Error::InvalidResult);
            }
            used[shipment] = true;
            visits.push(RouteVisit {
                shipment,
                kind,
                arrival: v.arrival_minute,
            });
        }
        routes.push(CourierRoute {
            courier,
            departure: r.depot_departure_minute,
            visits,
            depot_return: r.depot_return_minute,
        });
    }
    if elapsed != result.total_courier_minutes
        || !c
            .snapshot
            .shipments
            .iter()
            .enumerate()
            .filter(|(i, _)| !used[*i])
            .map(|(_, s)| &s.id)
            .eq(result.unassigned.iter())
    {
        return Err(Error::InvalidResult);
    }
    Ok(routes)
}
