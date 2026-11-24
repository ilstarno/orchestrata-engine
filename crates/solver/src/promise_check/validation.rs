use std::collections::HashSet;

use super::{PromiseError as E, PromiseRequest};

pub(super) fn validate(request: &PromiseRequest) -> Result<(), E> {
    if request.version != 1 {
        return Err(E::UnsupportedVersion);
    }
    let s = &request.snapshot;
    if s.jobs.len() > 200
        || s.resources.len() > 200
        || s.metrics.is_empty()
        || s.metrics.len() > 8
        || request.policies.is_empty()
        || request.policies.len() > 8
        || request.verification_work > 20_000_000
        || s.commitments.len() > s.jobs.len()
        || s.assumptions.len() > 16
        || s.assumptions
            .iter()
            .any(|v| v.len() > 256 || v.chars().any(char::is_control))
        || s.resources.iter().any(|r| r.capacity > 200)
    {
        return Err(E::LimitExceeded);
    }
    identity(&s.id)?;
    unique(s.metrics.iter().map(|v| v.id.as_str()))?;
    for metric in &s.metrics {
        identity(&metric.unit)?;
    }
    unique(s.resources.iter().map(|v| v.id.as_str()))?;
    unique(s.jobs.iter().map(|v| v.id.as_str()))?;
    unique(request.policies.iter().map(|v| v.id.as_str()))?;
    unique(s.commitments.iter().map(|v| v.job_id.as_str()))?;
    if s.incumbent.len() != s.jobs.len() {
        return Err(E::InvalidReference);
    }
    for job in &s.jobs {
        if job.options.len() > s.resources.len() {
            return Err(E::LimitExceeded);
        }
        unique(job.options.iter().map(|v| v.resource_id.as_str()))?;
        for option in &job.options {
            if !s.resources.iter().any(|r| r.id == option.resource_id) {
                return Err(E::InvalidReference);
            }
            if option.estimates.len() != s.metrics.len() {
                return Err(E::WrongEstimateCount);
            }
        }
    }
    for assigned in s.incumbent.iter().flatten() {
        if !s.resources.iter().any(|r| r.id == *assigned) {
            return Err(E::InvalidReference);
        }
    }
    for commitment in &s.commitments {
        let Some(job) = s.jobs.iter().position(|j| j.id == commitment.job_id) else {
            return Err(E::InvalidReference);
        };
        if s.incumbent[job].as_ref() != Some(&commitment.resource_id) {
            return Err(E::CommitmentConflict);
        }
    }
    for policy in &request.policies {
        if !s.metrics.iter().any(|m| m.id == policy.minimize_metric) {
            return Err(E::InvalidReference);
        }
    }
    Ok(())
}

fn unique<'a>(values: impl Iterator<Item = &'a str>) -> Result<(), E> {
    let mut seen = HashSet::new();
    for value in values {
        identity(value)?;
        if !seen.insert(value) {
            return Err(E::InvalidIdentity);
        }
    }
    Ok(())
}
fn identity(value: &str) -> Result<(), E> {
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-.:/".contains(&b))
    {
        return Err(E::InvalidIdentity);
    }
    Ok(())
}
