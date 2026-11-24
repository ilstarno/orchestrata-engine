/// Evidence performed for this result; no routing or other objective is implied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationScope {
    NoIncumbent,
    IndependentFeasibilityAndScore,
    IndependentFeasibilityAndScoreWithExhaustiveSearch,
    ExhaustiveInfeasibilitySearch,
}
