/// Supplied additive estimate, never an observed saving. Units are opaque;
/// v1 does not convert units, derive travel, or validate the estimation method.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EstimateMetric {
    pub id: String,
    pub unit: String,
}
