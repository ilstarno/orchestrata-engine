use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Constraint {
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
