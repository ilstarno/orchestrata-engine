use std::collections::HashSet;

/// An ordered finite domain. The first occurrence supplies the tie-break order.
pub(super) struct Domain(Vec<i64>);

impl Domain {
    pub(super) fn from_options(mut values: Vec<i64>) -> Self {
        if values.len() > 1 {
            let mut seen = HashSet::with_capacity(values.len());
            values.retain(|value| seen.insert(*value));
        }
        Self(values)
    }

    pub(super) fn into_options(self) -> Vec<i64> {
        self.0
    }
}
