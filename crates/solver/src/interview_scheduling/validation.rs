// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use std::collections::HashSet;
use std::hash::Hash;

use super::{AppointmentRequest, Interviewer, ScheduleError, TimeRange};

const MAX_REQUESTS: usize = 32;
const MAX_INTERVIEWERS: usize = 64;
const MAX_ROLES_PER_MEETING: usize = 4;
const MAX_STARTS_PER_MEETING: usize = 32;
const MAX_RESOURCE_ROLES: usize = 32;
const MAX_FREE_RANGES: usize = 64;

fn valid_id(id: &str) -> bool {
    (1..=64).contains(&id.len())
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
}

fn unique<T: Eq + Hash>(iter: impl Iterator<Item = T>) -> bool {
    let mut seen = HashSet::new();
    iter.into_iter().all(|id| seen.insert(id))
}

pub(super) fn occupied_range(
    request: &AppointmentRequest<'_>,
    start: u64,
) -> Result<TimeRange, ScheduleError> {
    Ok(TimeRange {
        start: start
            .checked_sub(u64::from(request.before))
            .ok_or(ScheduleError::TimeOverflow)?,
        end: start
            .checked_add(u64::from(request.duration) + u64::from(request.after))
            .ok_or(ScheduleError::TimeOverflow)?,
    })
}

pub(super) fn validate(
    requests: &[AppointmentRequest<'_>],
    interviewers: &[Interviewer<'_>],
) -> Result<(), ScheduleError> {
    if requests.len() > MAX_REQUESTS || interviewers.len() > MAX_INTERVIEWERS {
        return Err(ScheduleError::InputSizeLimit);
    }
    if !unique(requests.iter().map(|request| request.id))
        || !unique(interviewers.iter().map(|person| person.id))
    {
        return Err(ScheduleError::DuplicateIdentifier);
    }
    for person in interviewers {
        if person.roles.len() > MAX_RESOURCE_ROLES || person.free.len() > MAX_FREE_RANGES {
            return Err(ScheduleError::InputSizeLimit);
        }
        if !valid_id(person.id) || !person.roles.iter().copied().all(valid_id) {
            return Err(ScheduleError::InvalidIdentifier);
        }
        if !unique(person.roles.iter().copied()) {
            return Err(ScheduleError::DuplicateIdentifier);
        }
        if person.free.iter().any(|window| window.start >= window.end)
            || person
                .free
                .windows(2)
                .any(|pair| pair[0].end > pair[1].start)
        {
            return Err(ScheduleError::InvalidFreeRange);
        }
    }
    for request in requests {
        if request.roles.len() > MAX_ROLES_PER_MEETING
            || request.allowed_starts.len() > MAX_STARTS_PER_MEETING
        {
            return Err(ScheduleError::InputSizeLimit);
        }
        if request.roles.is_empty() || request.duration == 0 {
            return Err(ScheduleError::InvalidRequest);
        }
        if !valid_id(request.id) || !request.roles.iter().copied().all(valid_id) {
            return Err(ScheduleError::InvalidIdentifier);
        }
        if !unique(request.allowed_starts.iter().copied()) {
            return Err(ScheduleError::DuplicateIdentifier);
        }
        for &start in request.allowed_starts {
            occupied_range(request, start)?;
        }
        if let Some(previous) = request.previous {
            if previous.panel.len() != request.roles.len()
                || !unique(previous.panel.iter().copied())
                || !previous.panel.iter().copied().all(valid_id)
            {
                return Err(ScheduleError::InvalidPreviousPanel);
            }
        } else if request.locked {
            return Err(ScheduleError::LockedMeetingUnavailable);
        }
    }
    Ok(())
}
