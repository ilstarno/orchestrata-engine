/// Self-contained replay evidence. Deserialization is not validation:
/// call replay_decision before trusting a received bundle. No signature or
/// authenticity claim is provided; replay compares the complete typed result.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionBundle {
    pub version: u16,
    pub scope: super::DecisionScope,
    pub estimate_basis: super::EstimateBasis,
    pub request: super::PromiseRequest,
    pub incumbent: super::AssignmentPlan,
    pub comparisons: Vec<super::PolicyComparison>,
}
