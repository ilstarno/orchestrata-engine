/// Maximize assigned count, then minimize exactly this declared metric.
/// No objective weighting, minimum-change tie-break, routing or scheduling.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub id: String,
    pub minimize_metric: String,
}
