/// Evidence for maximum assigned count, then minimum supplied cost only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeightedEvidence {
    IndependentlyVerifiedFeasibleAndAccounted,
    ProvenOptimal,
}
