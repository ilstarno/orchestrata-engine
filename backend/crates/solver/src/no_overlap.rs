use std::collections::HashSet;

use crate::{Error, NoOverlapError, ScheduledInterval};

pub(super) fn validate_intervals(
    intervals: &[ScheduledInterval],
    variables: usize,
) -> Result<(), NoOverlapError> {
    if intervals.len() < 2 {
        return Err(NoOverlapError::TooFewIntervals);
    }
    let mut seen = HashSet::with_capacity(intervals.len());
    for interval in intervals {
        if interval.start >= variables {
            return Err(NoOverlapError::UnknownStart(interval.start));
        }
        if !seen.insert(interval.start) {
            return Err(NoOverlapError::DuplicateStart(interval.start));
        }
    }
    Ok(())
}

pub(super) fn has_overlap(
    intervals: &[ScheduledInterval],
    value_at: impl Fn(usize) -> i64,
) -> bool {
    let mut starts: Vec<_> = ranges(intervals, value_at).collect();
    starts.sort_unstable();
    starts.windows(2).any(|pair| pair[0].1 > pair[1].0)
}

/// Sum occupied overlap time across all unordered pairs without enumerating
/// pairs. Over every constant-active segment, k intervals contribute k(k-1)/2
/// pairs. Starts and ends at equal timestamps contribute no time.
pub(super) fn overlap_score(
    intervals: &[ScheduledInterval],
    value_at: impl Fn(usize) -> i64,
) -> Result<i128, Error> {
    let mut events: Vec<_> = ranges(intervals, value_at)
        .flat_map(|(start, end)| [(start, 1_i128), (end, -1_i128)])
        .collect();
    events.sort_unstable();

    let mut previous = events.first().map_or(0, |&(time, _)| time);
    let mut active = 0_i128;
    let mut penalty = 0_i128;
    for (time, delta) in events {
        if time > previous && active >= 2 {
            let pairs = active.checked_mul(active - 1).ok_or(Error::ScoreOverflow)? / 2;
            let overlap = (time - previous)
                .checked_mul(pairs)
                .ok_or(Error::ScoreOverflow)?;
            penalty = penalty.checked_sub(overlap).ok_or(Error::ScoreOverflow)?;
        }
        active += delta;
        previous = time;
    }
    Ok(penalty)
}

fn ranges(
    intervals: &[ScheduledInterval],
    value_at: impl Fn(usize) -> i64,
) -> impl Iterator<Item = (i128, i128)> {
    intervals
        .iter()
        .filter(|interval| interval.duration != 0)
        .map(move |interval| {
            let start = i128::from(value_at(interval.start));
            (start, start + i128::from(interval.duration))
        })
}
