/// A capacitated bipartite matching and equal-capacity cut.
/// Source-to-job capacity is one; job-to-slot capacity exceeds the job count.
/// `source_jobs` and `source_slots` mark the source side of the cut.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowWitness {
    pub eligible_slots: Vec<Vec<usize>>,
    pub slot_capacities: Vec<usize>,
    pub matching: Vec<Option<usize>>,
    pub source_jobs: Vec<bool>,
    pub source_slots: Vec<bool>,
    pub cut_capacity: usize,
}
