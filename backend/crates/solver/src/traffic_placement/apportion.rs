// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Assign whole units proportional to the weights using largest remainders.
/// The caller sorts target IDs first, so equal remainders go to ascending ID.
pub(super) fn apportion(weight: &[u64], total: u128, units: u64) -> Vec<u64> {
    let mut allocated = vec![0; weight.len()];
    if total == 0 || units == 0 {
        return allocated;
    }

    let mut left = units;
    let mut remainders = Vec::new();
    for (index, &weight) in weight.iter().enumerate() {
        let numerator = u128::from(weight) * u128::from(units);
        let whole = (numerator / total) as u64; // At most `units`.
        allocated[index] = whole;
        left -= whole;
        let fraction = numerator % total;
        if fraction != 0 {
            remainders.push((index, fraction));
        }
    }

    remainders.sort_unstable_by(|(left_index, left_part), (right_index, right_part)| {
        right_part
            .cmp(left_part)
            .then_with(|| left_index.cmp(right_index))
    });
    // The sum of all remainders is exactly `left * total`, so there are
    // always at least `left` positive fractions (and at most 256 targets).
    for &(index, _) in remainders.iter().take(left as usize) {
        allocated[index] += 1;
    }
    allocated
}
