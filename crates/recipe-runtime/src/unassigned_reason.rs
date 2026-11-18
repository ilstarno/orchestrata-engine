use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnassignedReason {
    NoMatchingSkill,
    MatchingWorkersUnavailable,
    NoAvailableCapacity,
    CapacityExhausted,
}
