use serde::{Deserialize, Serialize};

use super::count_value_tier::CountValueTier;

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
    PreferValue {
        variable: usize,
        value: i64,
        soft_weight: i64,
    },
}
