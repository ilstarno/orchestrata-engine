/// Distance outside an inclusive, validated range.
pub(super) fn distance(count: usize, min: usize, max: usize) -> usize {
    min.saturating_sub(count) + count.saturating_sub(max)
}
