// SPDX-License-Identifier: Apache-2.0

use std::collections::HashMap;

use crate::{ResourceLimit, ResourceLoad};

/// Loads for one evaluated resource constraint. Storage may be reused.
#[derive(Default)]
pub(super) struct ResourceLoadTotals {
    by_resource: HashMap<i64, u128>,
}

impl ResourceLoadTotals {
    pub(super) fn collect(&mut self, items: &[ResourceLoad], value_at: impl Fn(usize) -> i64) {
        self.by_resource.clear();
        for item in items {
            *self.by_resource.entry(value_at(item.variable)).or_default() += u128::from(item.load);
        }
    }

    pub(super) fn overload(&self, limit: &ResourceLimit) -> u128 {
        self.by_resource
            .get(&limit.resource)
            .copied()
            .unwrap_or_default()
            .saturating_sub(u128::from(limit.capacity))
    }
}
