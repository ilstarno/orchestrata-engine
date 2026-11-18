use serde::Serialize;
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ModelAssignment {
    pub item_id: String,
    pub resource_id: Option<String>,
}
