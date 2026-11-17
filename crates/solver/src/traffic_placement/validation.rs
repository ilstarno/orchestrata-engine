// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use std::collections::HashSet;

use super::{
    CapacityEnvelope, Health, HealthSnapshot, StalenessPolicy, TrafficError, UnusableReason,
};

pub(super) const MAX_TARGETS: usize = 256;
const MAX_REGIONS: usize = 16;

pub(super) fn identifier(id: &str) -> Result<(), TrafficError> {
    if (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, b'-' | b'_' | b'.' | b':'))
    {
        Ok(())
    } else {
        Err(TrafficError::InvalidIdentifier)
    }
}

pub(super) fn regions(ordered_regions: &[&str]) -> Result<(), TrafficError> {
    if ordered_regions.len() > MAX_REGIONS {
        return Err(TrafficError::InputSizeLimit);
    }
    if ordered_regions.is_empty() {
        return Err(TrafficError::EmptyApprovedRegions);
    }
    let mut approved = HashSet::with_capacity(ordered_regions.len());
    for &region in ordered_regions {
        identifier(region)?;
        if !approved.insert(region) {
            return Err(TrafficError::DuplicateRegion);
        }
    }
    Ok(())
}

pub(super) fn targets<'a>(
    count: usize,
    input: impl Iterator<Item = (&'a str, &'a str, HealthSnapshot)>,
    staleness: StalenessPolicy,
) -> Result<(), TrafficError> {
    if count > MAX_TARGETS {
        return Err(TrafficError::InputSizeLimit);
    }
    let mut seen = HashSet::with_capacity(count);
    for (id, scope, observation) in input {
        identifier(id)?;
        identifier(scope)?;
        if !seen.insert(id) {
            return Err(TrafficError::DuplicateEndpoint);
        }
        if observation
            .checked_at_seconds
            .is_some_and(|checked| checked > staleness.evaluated_at_seconds)
        {
            return Err(TrafficError::FutureHealthCheck);
        }
    }
    Ok(())
}

pub(super) fn usable_capacity(
    admitted: bool,
    observation: HealthSnapshot,
    capacity: CapacityEnvelope,
    policy: StalenessPolicy,
) -> Result<u64, UnusableReason> {
    if !admitted {
        return Err(UnusableReason::NotAdmitted);
    }
    match observation.status {
        Health::Unhealthy => return Err(UnusableReason::ReportedUnhealthy),
        Health::Unknown => return Err(UnusableReason::UnknownHealth),
        Health::Healthy => {}
    }
    let checked = observation
        .checked_at_seconds
        .ok_or(UnusableReason::MissingHealthCheck)?;
    if policy.evaluated_at_seconds.saturating_sub(checked) > policy.maximum_age_seconds {
        return Err(UnusableReason::StaleHealthCheck);
    }

    let available = capacity.headroom();
    if available == 0 {
        return Err(UnusableReason::NoHeadroom);
    }
    Ok(available)
}
