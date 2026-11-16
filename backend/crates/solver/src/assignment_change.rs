use crate::AssignmentChangeReason;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignmentChange {
    /// Position in the variable order shared by both models.
    pub variable_index: usize,
    pub previous_value: i64,
    pub updated_value: i64,
    pub reason: AssignmentChangeReason,
}
