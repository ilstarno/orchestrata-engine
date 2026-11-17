use std::{error::Error, fmt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapacityMatchError {
    TooManyRecords,
    InvalidCapacity { worker: usize },
    UnknownWorker { job: usize, worker: usize },
    RepeatedWorker { job: usize, worker: usize },
}

impl fmt::Display for CapacityMatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyRecords => f.write_str("At most 200 jobs and 200 workers are supported."),
            Self::InvalidCapacity { worker } => {
                write!(f, "Worker {worker} has a capacity above 200.")
            }
            Self::UnknownWorker { job, worker } => {
                write!(f, "Job {job} refers to unknown worker {worker}.")
            }
            Self::RepeatedWorker { job, worker } => {
                write!(f, "Job {job} lists worker {worker} more than once.")
            }
        }
    }
}

impl Error for CapacityMatchError {}
