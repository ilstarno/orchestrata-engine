/// No routing, temporal scheduling or horizon-wide conclusion.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum DecisionScope {
    SingleSnapshotCapacityAssignment,
}
