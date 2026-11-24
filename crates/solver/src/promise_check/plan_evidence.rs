/// Optimality is conditional on fixed commitments, declared eligibility,
/// job-count capacity and one supplied metric, on this snapshot only.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum PlanEvidence {
    VerifiedFeasibleOptimalityUnknown,
    ProvenOptimalUnderCommitments,
}
