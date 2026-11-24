#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ChangeKind {
    Assigned,
    Moved,
    Unassigned,
}
