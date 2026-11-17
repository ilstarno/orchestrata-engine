use std::collections::HashMap;

use crate::{Error, ResourceLimit, ResourceLoad};

pub(super) fn exceeds_capacity(
    items: &[ResourceLoad],
    limits: &[ResourceLimit],
    value_at: impl Fn(usize) -> i64,
) -> bool {
    let totals = assigned_loads(items, value_at);
    limits.iter().any(|limit| {
        totals.get(&limit.resource).copied().unwrap_or_default() > u128::from(limit.capacity)
    })
}

pub(super) fn overload_penalty(
    items: &[ResourceLoad],
    limits: &[ResourceLimit],
    value_at: impl Fn(usize) -> i64,
) -> Result<i128, Error> {
    let totals = assigned_loads(items, value_at);
    limits.iter().try_fold(0_i128, |penalty, limit| {
        let overload = totals
            .get(&limit.resource)
            .copied()
            .unwrap_or_default()
            .saturating_sub(u128::from(limit.capacity));
        let overload = i128::try_from(overload).map_err(|_| Error::ScoreOverflow)?;
        penalty.checked_add(overload).ok_or(Error::ScoreOverflow)
    })
}

fn assigned_loads(items: &[ResourceLoad], value_at: impl Fn(usize) -> i64) -> HashMap<i64, u128> {
    let mut totals = HashMap::new();
    for item in items {
        *totals.entry(value_at(item.variable)).or_default() += u128::from(item.load);
    }
    totals
}
