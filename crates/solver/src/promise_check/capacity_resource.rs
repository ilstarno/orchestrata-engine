/// Capacity counts whole jobs within the supplied snapshot, not hours.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapacityResource {
    pub id: String,
    pub capacity: usize,
}
