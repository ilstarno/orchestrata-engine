/// Deltas are proposed minus incumbent: negative estimate deltas are modeled
/// reductions, not observed savings. No cause is inferred for unassigned work.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyComparison {
    pub policy_id: String,
    pub plan: super::AssignmentPlan,
    pub changes: Vec<super::PlanChange>,
    pub unassigned_job_ids: Vec<String>,
    pub assigned_delta: i64,
    pub estimate_deltas: Vec<i64>,
    pub evidence: super::PlanEvidence,
    pub verification: super::VerificationOutcome,
}
