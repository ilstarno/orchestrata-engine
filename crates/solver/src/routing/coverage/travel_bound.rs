use super::{
    super::{CourierMode, CourierRoute, LocationId, RoutingProblem},
    CourierTravelCapacity, CoverageError, CoverageOutcome, TravelCoverageCertificate,
    TravelCoverageOutcome,
    budget::Budget,
    certify_coverage, flow,
    relaxation::Relaxation,
};
use std::collections::BTreeMap;

/// Explicit stronger coverage bound for the same static directed routing model.
/// Includes the original certificate's work in the caller's total budget.
/// Recomputes shortest paths; no direct-arc exclusion or travel optimality claim.
pub fn certify_coverage_with_travel(
    p: &RoutingProblem,
    incumbent: &[CourierRoute],
    max_work: u32,
    travel: impl Fn(CourierMode, LocationId, LocationId) -> Option<u64>,
) -> Result<TravelCoverageOutcome, CoverageError> {
    let base = match certify_coverage(p, incumbent, max_work, &travel)? {
        CoverageOutcome::Complete(base) => *base,
        CoverageOutcome::Incomplete { work } => {
            return Ok(TravelCoverageOutcome::Incomplete { work });
        }
    };
    let mut b = Budget {
        used: base.work,
        limit: max_work,
    };
    let mut compute = || -> Result<_, ()> {
        b.step()?;
        let relaxed = Relaxation::new(p, &travel, &mut b)?;
        let mut bounds = Vec::new();
        for c in 0..p.couriers.len() {
            bounds.push(capacity(
                p,
                c,
                &base.service.eligible_slots,
                &relaxed,
                &mut b,
            )?);
        }
        let mut edges = Vec::new();
        for row in &base.service.eligible_slots {
            b.step()?;
            edges.push(row.clone());
        }
        let mut caps = Vec::new();
        for bound in &bounds {
            b.step()?;
            caps.push(bound.capacity);
        }
        Ok((flow::solve(edges, caps, &mut b)?, bounds))
    };
    let Ok((witness, bounds)) = compute() else {
        return Ok(TravelCoverageOutcome::Incomplete { work: b.used });
    };
    if !flow::verify_flow_witness(&witness) {
        return Err(CoverageError::InconsistentBound);
    }
    let upper_bound = base.upper_bound.min(witness.cut_capacity);
    let covered = base.incumbent_covered;
    if upper_bound < covered {
        return Err(CoverageError::InconsistentBound);
    }
    Ok(TravelCoverageOutcome::Complete(Box::new(
        TravelCoverageCertificate {
            base,
            travel: witness,
            courier_bounds: bounds,
            upper_bound,
            incumbent_covered: covered,
            maximum_coverage: covered == upper_bound,
            work: b.used,
        },
    )))
}

fn capacity(
    p: &RoutingProblem,
    c: usize,
    edges: &[Vec<usize>],
    r: &Relaxation,
    b: &mut Budget,
) -> Result<CourierTravelCapacity, ()> {
    let courier = &p.couriers[c];
    let mut groups = BTreeMap::new();
    let mut depot_only = 0usize;
    let mut services = Vec::new();
    for (j, row) in edges.iter().enumerate() {
        b.step()?;
        if !row.contains(&c) {
            continue;
        }
        let s = &p.shipments[j];
        services.push(
            u128::from(s.pickup.service_time)
                .checked_add(u128::from(s.delivery.service_time))
                .ok_or(())?,
        );
        let representative = if s.delivery.location != courier.depot {
            s.delivery.location
        } else {
            s.pickup.location
        };
        if representative == courier.depot {
            depot_only += 1;
        } else {
            *groups.entry(representative.0).or_insert(0usize) += 1;
        }
    }
    let mut representatives = Vec::new();
    for (location, count) in groups {
        b.step()?;
        representatives.push((LocationId(location), count));
    }
    let mut outbound = None;
    let mut inbound = None;
    let mut between = None;
    for &(location, _) in &representatives {
        b.step()?;
        minimize(
            &mut outbound,
            r.shortest(courier.mode, courier.depot, location)
                .ok_or(())?,
        );
        b.step()?;
        minimize(
            &mut inbound,
            r.shortest(courier.mode, location, courier.depot)
                .ok_or(())?,
        );
        for &(other, _) in &representatives {
            b.step()?;
            if location != other {
                minimize(
                    &mut between,
                    r.shortest(courier.mode, location, other).ok_or(())?,
                );
            }
        }
    }
    // Selection scans charge each comparison; no unbudgeted sort or search.
    let mut prefixes = vec![0u128];
    while !services.is_empty() {
        let mut best = 0;
        for i in 0..services.len() {
            b.step()?;
            if services[i] < services[best] {
                best = i;
            }
        }
        let value = services.swap_remove(best);
        prefixes.push(
            prefixes
                .last()
                .copied()
                .ok_or(())?
                .checked_add(value)
                .ok_or(())?,
        );
    }
    let mut multiplicities = Vec::new();
    for &(_, count) in &representatives {
        b.step()?;
        multiplicities.push(count);
    }
    let mut admitted = vec![depot_only];
    while !multiplicities.is_empty() {
        let mut best = 0;
        for i in 0..multiplicities.len() {
            b.step()?;
            if multiplicities[i] > multiplicities[best] {
                best = i;
            }
        }
        let count = multiplicities.swap_remove(best);
        admitted.push(
            admitted
                .last()
                .copied()
                .ok_or(())?
                .checked_add(count)
                .ok_or(())?,
        );
    }
    let mut distinct_nodes = Vec::new();
    let mut travel_lower_bounds = Vec::new();
    let mut capacity = 0;
    for (k, &service) in prefixes.iter().enumerate() {
        b.step()?;
        let mut d = 0;
        while *admitted.get(d).ok_or(())? < k {
            b.step()?;
            d += 1;
        }
        let travel = if d == 0 {
            0
        } else {
            let ends = outbound
                .ok_or(())?
                .checked_add(inbound.ok_or(())?)
                .ok_or(())?;
            if d == 1 {
                ends
            } else {
                ends.checked_add(between.ok_or(())?.checked_mul((d - 1) as u128).ok_or(())?)
                    .ok_or(())?
            }
        };
        distinct_nodes.push(d);
        travel_lower_bounds.push(travel);
        if service.checked_add(travel).ok_or(())?
            <= u128::from(courier.shift.end - courier.shift.start)
        {
            capacity = k;
        }
    }
    Ok(CourierTravelCapacity {
        courier: c,
        capacity,
        depot_only,
        representatives,
        minimum_outbound: outbound,
        minimum_return: inbound,
        minimum_between: between,
        service_prefixes: prefixes,
        distinct_nodes,
        travel_lower_bounds,
    })
}
fn minimize(slot: &mut Option<u128>, value: u128) {
    *slot = Some(slot.map_or(value, |old| old.min(value)));
}
