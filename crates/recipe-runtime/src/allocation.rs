use crate::UnassignedReason;
use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct Allocation {
    pub job_id: String,
    pub worker_id: Option<String>,
    pub unassigned_reason: Option<UnassignedReason>,
}
