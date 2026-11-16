// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{
    AllocationRequest, Backend, BackendAllocation, BackendDisposition, BackendShare, TrafficError,
    apportion::apportion,
    validation::{identifier, targets, usable_capacity},
};

/// Spread additional load within an explicitly requested backend pool.
///
/// Maximize served demand subject to commitments and reserve, then distribute
/// that load in proportion to each usable backend's remaining capacity. The
/// proposal is an integer plan for one caller-defined interval, not live load.
pub fn plan_backend_allocation<'a>(
    request: AllocationRequest<'a>,
    backends: &[Backend<'a>],
) -> Result<BackendAllocation<'a>, TrafficError> {
    identifier(request.pool)?;
    targets(
        backends.len(),
        backends.iter().map(|b| (b.id, b.pool, b.observation)),
        request.staleness,
    )?;

    let mut backends = backends.to_vec();
    backends.sort_unstable_by_key(|backend| backend.id);
    let candidates: Vec<_> = backends
        .iter()
        .map(|backend| {
            if backend.pool != request.pool {
                return Err(BackendDisposition::OutsidePool);
            }
            usable_capacity(
                backend.admitted,
                backend.observation,
                backend.capacity,
                request.staleness,
            )
            .map_err(BackendDisposition::Unusable)
        })
        .collect();
    let headroom: Vec<_> = candidates
        .iter()
        .map(|candidate| candidate.unwrap_or(0))
        .collect();
    let available: u128 = headroom.iter().map(|&value| u128::from(value)).sum();
    let served = available.min(u128::from(request.demand)) as u64;
    let load = apportion(&headroom, available, served);
    let backend_shares = backends
        .iter()
        .zip(candidates)
        .zip(load)
        .map(|((backend, candidate), proposed)| BackendShare {
            backend_id: backend.id,
            available: candidate.unwrap_or(0),
            proposed,
            disposition: match candidate {
                Err(reason) => reason,
                Ok(_) if proposed == 0 => BackendDisposition::NoLoadProposed,
                Ok(_) => BackendDisposition::LoadProposed,
            },
        })
        .collect();
    Ok(BackendAllocation {
        demand: request.demand,
        served,
        unserved: request.demand - served,
        available,
        backend_shares,
    })
}
