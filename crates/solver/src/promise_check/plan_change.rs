#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanChange {
    pub job_id: String,
    pub previous: Option<String>,
    pub proposed: Option<String>,
    pub kind: super::ChangeKind,
}
