use crate::{Error, ResourceLimit, ResourceLoad};

use super::resource_load_totals::ResourceLoadTotals;

pub(super) fn overload_penalty(
    items: &[ResourceLoad],
    limits: &[ResourceLimit],
    value_at: impl Fn(usize) -> i64,
) -> Result<i128, Error> {
    let mut totals = ResourceLoadTotals::default();
    totals.collect(items, value_at);
    limits.iter().try_fold(0_i128, |penalty, limit| {
        let overload = i128::try_from(totals.overload(limit)).map_err(|_| Error::ScoreOverflow)?;
        penalty.checked_add(overload).ok_or(Error::ScoreOverflow)
    })
}
