use super::ModelAssignment;
use serde::Serialize;
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct PenaltyModelSolution {
    pub schema_version: u32,
    pub objective: &'static str,
    pub unit: String,
    pub assigned: usize,
    pub service_cost: u64,
    pub unassigned_cost: u64,
    pub total_cost: u64,
    pub assignments: Vec<ModelAssignment>,
}
