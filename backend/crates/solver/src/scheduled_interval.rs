use serde::{Deserialize, Serialize};

/// A fixed-duration interval whose start is a zero-based variable reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ScheduledInterval {
    pub start: usize,
    pub duration: u64,
}
