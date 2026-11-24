/// One immutable planning snapshot shared by all policies.
/// Incumbent entries follow jobs order; None means explicitly unassigned.
/// The caller's id is a label, not a cryptographic identity or freshness proof.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanningSnapshot {
    pub id: String,
    pub metrics: Vec<super::EstimateMetric>,
    pub resources: Vec<super::CapacityResource>,
    pub jobs: Vec<super::WorkItem>,
    pub incumbent: Vec<Option<String>>,
    pub commitments: Vec<super::Commitment>,
    pub assumptions: Vec<String>,
}
