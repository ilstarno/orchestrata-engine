/// Promise Check v1. Typed consumers need not use JSON.
/// Verification work is a per-policy residual-arc budget, not a solve deadline.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromiseRequest {
    pub version: u16,
    pub snapshot: super::PlanningSnapshot,
    pub policies: Vec<super::Policy>,
    pub verification_work: u64,
}
