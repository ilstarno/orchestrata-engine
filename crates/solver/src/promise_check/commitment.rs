/// Must match a feasible incumbent assignment. Never silently relaxed.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commitment {
    pub job_id: String,
    pub resource_id: String,
}
