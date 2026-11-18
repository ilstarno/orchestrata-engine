use crate::Allocation;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct AllocationResult {
    pub assigned: usize,
    pub output_field: String,
    pub allocations: Vec<Allocation>,
}
