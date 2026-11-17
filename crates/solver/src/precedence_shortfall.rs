/// All combinations of signed times and an unsigned gap fit in this widened calculation.
pub(super) fn shortfall(before: i64, after: i64, min_gap: u64) -> i128 {
    (i128::from(before) + i128::from(min_gap) - i128::from(after)).max(0)
}
