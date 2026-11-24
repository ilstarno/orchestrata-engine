/// Explicit eligible assignment. Estimates follow snapshot.metrics order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AssignmentOption {
    pub resource_id: String,
    pub estimates: Vec<u32>,
}
