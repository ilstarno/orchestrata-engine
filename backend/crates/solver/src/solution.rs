use serde::{Deserialize, Serialize};

use crate::Score;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
pub struct Solution {
    pub values: Vec<i64>,
    pub score: Score,
    pub evaluated: u64,
}
