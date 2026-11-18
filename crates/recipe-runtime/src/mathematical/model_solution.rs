use super::model_assignment::ModelAssignment;
use serde::Serialize;
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ModelSolution {
    pub assigned: usize,
    pub minimized_edge_sum: Option<u64>,
    pub assignments: Vec<ModelAssignment>,
}
