use crate::{SolutionEvidence, SolveObjective, SolveTermination, VerificationScope};

/// Native-only reference-search result. Fields cannot be mutated into conflicting
/// claims. No serialization or general-purpose objective gap is supplied.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SolveReport {
    termination: SolveTermination,
    evidence: SolutionEvidence,
    evaluated: u64,
}

impl SolveReport {
    pub(super) fn new(
        termination: SolveTermination,
        evidence: SolutionEvidence,
        evaluated: u64,
    ) -> Self {
        Self {
            termination,
            evidence,
            evaluated,
        }
    }

    pub(super) fn into_evidence(self) -> SolutionEvidence {
        self.evidence
    }

    pub fn termination(&self) -> SolveTermination {
        self.termination
    }
    pub fn evidence(&self) -> &SolutionEvidence {
        &self.evidence
    }
    pub fn evaluated(&self) -> u64 {
        self.evaluated
    }
    pub fn objective(&self) -> SolveObjective {
        SolveObjective::MaximizeSoftScoreSubjectToHardConstraints
    }
    pub fn verification_scope(&self) -> VerificationScope {
        match self.evidence {
            SolutionEvidence::Unknown => VerificationScope::NoIncumbent,
            SolutionEvidence::VerifiedFeasible(_) => {
                VerificationScope::IndependentFeasibilityAndScore
            }
            SolutionEvidence::ProvenOptimal(_) => {
                VerificationScope::IndependentFeasibilityAndScoreWithExhaustiveSearch
            }
            SolutionEvidence::ProvenInfeasible => VerificationScope::ExhaustiveInfeasibilitySearch,
        }
    }
    /// Exact objective value only after exhaustive completion with a checked
    /// incumbent. No bound is inferred from a budget-limited incumbent.
    pub fn proven_optimal_soft_score(&self) -> Option<i64> {
        match &self.evidence {
            SolutionEvidence::ProvenOptimal(solution) => Some(solution.score.soft),
            _ => None,
        }
    }
}
