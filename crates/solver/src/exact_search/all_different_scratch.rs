// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Reuses storage for wide exact-search scopes; does not alter reported scores.
#[derive(Default)]
pub(super) struct AllDifferentScratch {
    values: Vec<i64>,
}

impl AllDifferentScratch {
    pub(super) fn has_duplicate(&mut self, scope: &[usize], values: &[i64]) -> bool {
        if scope.len() <= 8 {
            return scope.iter().enumerate().any(|(i, &left)| {
                scope[i + 1..]
                    .iter()
                    .any(|&right| values[left] == values[right])
            });
        }
        // Keep the first pairwise scan's early exit for common repeated values.
        // Its value does not need to enter the sorted scratch buffer.
        let first = values[scope[0]];
        let remaining = &scope[1..];
        if remaining.iter().any(|&index| values[index] == first) {
            return true;
        }
        self.values.clear();
        self.values
            .extend(remaining.iter().map(|&index| values[index]));
        self.values.sort_unstable();
        self.values.windows(2).any(|pair| pair[0] == pair[1])
    }
}
