use serde::{Deserialize, Serialize};

/// Nonnegative load contributed by one variable to its selected resource.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "openapi", derive(utoipa::ToSchema))]
#[serde(deny_unknown_fields)]
pub struct ResourceLoad {
    pub variable: usize,
    pub load: u64,
}
