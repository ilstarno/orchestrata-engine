/// An eligible zero-based worker index and a nonnegative cost for this job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkerCost {
    pub worker: usize,
    pub cost: u32,
}
