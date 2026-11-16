use serde::{Deserialize, Serialize};

/// Score component affected by a `count_value` rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(rename_all = "snake_case")]
pub enum CountValueTier {
    Hard,
    Soft,
}
