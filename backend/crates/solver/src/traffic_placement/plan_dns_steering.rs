// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{
    DnsDisposition, DnsEndpoint, DnsPolicy, DnsRequest, DnsWeight, TrafficError,
    apportion::apportion,
    validation::{regions, targets, usable_capacity},
};

/// Unitless review weights; do not assume a provider accepts this scale.
pub const DNS_WEIGHT_SCALE: u16 = 1000;

/// Propose relative DNS weights for the first eligible approved region.
///
/// A target qualifies only if admitted, explicitly healthy with a current
/// observation, and has positive capacity after commitments and reserve.
/// Only the earliest region with one or more qualifying targets is used.
/// Weights sum to 1000 in that region, or zero if none qualifies.
pub fn plan_dns_steering<'a>(
    request: DnsRequest<'a>,
    endpoints: &[DnsEndpoint<'a>],
) -> Result<DnsPolicy<'a>, TrafficError> {
    regions(request.approved_regions)?;
    targets(
        endpoints.len(),
        endpoints.iter().map(|e| (e.id, e.region, e.observation)),
        request.staleness,
    )?;

    let mut endpoints = endpoints.to_vec();
    endpoints.sort_unstable_by_key(|endpoint| endpoint.id);
    let candidates: Vec<_> = endpoints
        .iter()
        .map(|endpoint| {
            let region = request
                .approved_regions
                .iter()
                .position(|&approved| approved == endpoint.region)
                .ok_or(DnsDisposition::RegionNotApproved)?;
            let capacity = usable_capacity(
                endpoint.admitted,
                endpoint.observation,
                endpoint.capacity,
                request.staleness,
            )
            .map_err(DnsDisposition::Unusable)?;
            Ok((region, capacity))
        })
        .collect();
    let region = candidates
        .iter()
        .filter_map(|candidate| candidate.as_ref().ok().map(|&(rank, _)| rank))
        .min();
    let headroom: Vec<_> = candidates
        .iter()
        .map(|candidate| match candidate {
            Ok((rank, capacity)) if Some(*rank) == region => *capacity,
            _ => 0,
        })
        .collect();
    let total = headroom.iter().map(|&weight| u128::from(weight)).sum();
    let weights = apportion(&headroom, total, u64::from(DNS_WEIGHT_SCALE));
    let endpoint_weights = endpoints
        .iter()
        .zip(candidates)
        .zip(weights)
        .map(|((endpoint, candidate), weight)| DnsWeight {
            endpoint_id: endpoint.id,
            weight: weight as u16, // Bounded by `DNS_WEIGHT_SCALE`.
            disposition: match candidate {
                Err(reason) => reason,
                Ok((rank, _)) if Some(rank) != region => DnsDisposition::EarlierRegionAvailable,
                Ok(_) if weight == 0 => DnsDisposition::BelowWeightResolution,
                Ok(_) => DnsDisposition::WeightProposed,
            },
        })
        .collect();
    Ok(DnsPolicy {
        selected_region: region.map(|index| request.approved_regions[index]),
        endpoint_weights,
    })
}
