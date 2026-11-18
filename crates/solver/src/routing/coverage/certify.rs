use super::{
    super::{
        CourierMode, CourierRoute, LocationId, RoutingProblem, verification::validate_problem,
    },
    CourierClique, CoverageCertificate, CoverageError as Error, CoverageOutcome, MAX_COVERAGE_WORK,
    budget::Budget,
    flow,
    relaxation::Relaxation,
    replay::replay,
};

/// Opt-in maximum-coverage bound for static, repeatable directed travel.
/// Replays the incumbent with original arcs, then computes two optimistic flow
/// relaxations. Shortest paths only use problem stop/depot locations, which
/// include every location an actual route can visit. No travel objective is
/// certified. The work budget counts closure, relaxation and flow steps;
/// bounded input validation and incumbent replay are outside that counter.
pub fn certify_coverage(
    p: &RoutingProblem,
    incumbent: &[CourierRoute],
    max_work: u32,
    travel: impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Result<CoverageOutcome, Error> {
    if p.couriers.len() > 16
        || p.shipments.len() > 32
        || max_work > MAX_COVERAGE_WORK
        || p.shipments.iter().any(|s| s.allowed_modes.len() > 3)
    {
        return Err(Error::LimitExceeded);
    }
    validate_problem(p).map_err(Error::InvalidProblem)?;
    let covered = replay(p, incumbent, &travel)?;
    let mut budget = Budget {
        used: 0,
        limit: max_work,
    };
    let result = compute(p, &travel, &mut budget);
    let Ok((service, conflicts, cliques)) = result else {
        return Ok(CoverageOutcome::Incomplete { work: budget.used });
    };
    if !flow::verify_flow_witness(&service) || !flow::verify_flow_witness(&conflicts) {
        return Err(Error::InconsistentBound);
    }
    let upper_bound = service.cut_capacity.min(conflicts.cut_capacity);
    if covered > upper_bound {
        return Err(Error::InconsistentBound);
    }
    Ok(CoverageOutcome::Complete(Box::new(CoverageCertificate {
        upper_bound,
        incumbent_covered: covered,
        maximum_coverage: covered == upper_bound,
        work: budget.used,
        service,
        conflicts,
        cliques,
    })))
}
fn compute(
    p: &RoutingProblem,
    travel: &impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
    b: &mut Budget,
) -> Result<(super::FlowWitness, super::FlowWitness, Vec<CourierClique>), ()> {
    b.step()?;
    let r = Relaxation::new(p, travel, b)?;
    let n = p.shipments.len();
    let mut service_edges = vec![Vec::new(); n];
    let mut service_caps = Vec::new();
    let mut clique_edges = vec![Vec::new(); n];
    let mut cliques: Vec<CourierClique> = Vec::new();
    for (c, courier) in p.couriers.iter().enumerate() {
        let first = cliques.len();
        let mut service_times = Vec::new();
        for j in 0..n {
            if !r.feasible(p, c, &[(j, true), (j, false)], b)? {
                continue;
            }
            service_edges[j].push(c);
            let s = &p.shipments[j];
            service_times
                .push(u128::from(s.pickup.service_time) + u128::from(s.delivery.service_time));
            let mut found = None;
            for (slot, clique) in cliques.iter().enumerate().skip(first) {
                let mut all = true;
                for &other in &clique.shipments {
                    if !r.conflict(p, c, j, other, b)? {
                        all = false;
                        break;
                    }
                }
                if all {
                    found = Some(slot);
                    break;
                }
            }
            let slot = found.unwrap_or(cliques.len());
            if found.is_none() {
                cliques.push(CourierClique {
                    courier: c,
                    shipments: Vec::new(),
                });
            }
            cliques[slot].shipments.push(j);
            clique_edges[j].push(slot);
        }
        service_times.sort_unstable();
        let mut sum = 0;
        let mut count = 0;
        for t in service_times {
            b.step()?;
            sum += t;
            if sum > u128::from(courier.shift.end - courier.shift.start) {
                break;
            }
            count += 1;
        }
        service_caps.push(count);
    }
    let service = flow::solve(service_edges, service_caps, b)?;
    let conflicts = flow::solve(clique_edges, vec![1; cliques.len()], b)?;
    Ok((service, conflicts, cliques))
}
