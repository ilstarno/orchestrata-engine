use serde::{Deserialize, Serialize};

use super::count_value_tier::CountValueTier;
use crate::{ResourceLimit, ResourceLoad, ScheduledInterval};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
    /// Count selected variables equal to one value; penalize each unit outside the range.
    CountValue {
        variables: Vec<usize>,
        value: i64,
        min: usize,
        max: usize,
        weight: i64,
        tier: CountValueTier,
    },
    AllDifferent {
        variables: Vec<usize>,
        hard_weight: i64,
    },
    NotEqualOffset {
        left: usize,
        right: usize,
        offset: i64,
        hard_weight: i64,
    },
    /// Requires `after >= before + min_gap` in a consistent caller-chosen unit.
    /// The hard penalty is the positive shortfall in that unit.
    Precedence {
        before: usize,
        after: usize,
        min_gap: u64,
    },
    /// Requires disjoint half-open intervals on one exclusive resource.
    NoOverlap { intervals: Vec<ScheduledInterval> },
    /// Limits the sum of assigned item loads per resource in one unit/dimension.
    /// The hard penalty is the total amount above the resource limits.
    ResourceCapacity {
        items: Vec<ResourceLoad>,
        limits: Vec<ResourceLimit>,
    },
    PreferValue {
        variable: usize,
        value: i64,
        soft_weight: i64,
    },
}
