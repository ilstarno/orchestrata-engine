/// Assignments follow snapshot.jobs; totals follow snapshot.metrics.
/// Totals sum assigned work only; compare coverage alongside these estimates.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentPlan {
    pub assignments: Vec<Option<String>>,
    pub assigned: usize,
    pub estimate_totals: Vec<u64>,
}
