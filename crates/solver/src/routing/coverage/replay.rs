use super::{
    super::{
        CourierMode, CourierRoute, LocationId, RouteVerificationError, RoutingProblem,
        verify_routes,
    },
    CoverageError as Error,
};
pub(super) fn replay(
    problem: &RoutingProblem,
    routes: &[CourierRoute],
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Result<usize, Error> {
    if routes.len() > 16 || routes.iter().map(|r| r.visits.len()).sum::<usize>() > 64 {
        return Err(Error::LimitExceeded);
    }
    let mut selected = vec![false; problem.shipments.len()];
    for v in routes.iter().flat_map(|r| &r.visits) {
        *selected.get_mut(v.shipment).ok_or(Error::InvalidIncumbent(
            RouteVerificationError::UnknownShipment(v.shipment),
        ))? = true;
    }
    let mut mapping = vec![0; selected.len()];
    let mut shipments = Vec::new();
    for (i, &used) in selected.iter().enumerate() {
        if used {
            mapping[i] = shipments.len();
            shipments.push(problem.shipments[i].clone());
        }
    }
    let covered = shipments.len();
    let projection = RoutingProblem {
        couriers: problem.couriers.clone(),
        shipments,
    };
    let mut remapped = routes.to_vec();
    for v in remapped.iter_mut().flat_map(|r| &mut r.visits) {
        v.shipment = mapping[v.shipment];
    }
    verify_routes(&projection, &remapped, |m, f, t, _| travel(m, f, t))
        .map_err(Error::InvalidIncumbent)?;
    Ok(covered)
}
