use serde::{Deserialize, Serialize};

/// Inclusive capacity for a signed resource ID in the item variables' domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ResourceLimit {
    pub resource: i64,
    pub capacity: u64,
}
