use super::{
    CertificateEvaluation, WeightedEvidence, WeightedMatchAssignment, WeightedObjective,
    WeightedTermination,
};

/// Immutable native result, scoped to the exact weighted assignment input.
/// No serialization or general objective gap is supplied. Proof applies only
/// to maximum coverage followed by minimum supplied cost on this exact input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeightedReport {
    assignment: WeightedMatchAssignment,
    termination: WeightedTermination,
    certificate_evaluation: CertificateEvaluation,
}
impl WeightedReport {
    pub(super) fn new(
        assignment: WeightedMatchAssignment,
        termination: WeightedTermination,
        certificate_evaluation: CertificateEvaluation,
    ) -> Self {
        Self {
            assignment,
            termination,
            certificate_evaluation,
        }
    }
    pub fn assignment(&self) -> &WeightedMatchAssignment {
        &self.assignment
    }
    pub fn termination(&self) -> WeightedTermination {
        self.termination
    }
    pub fn evidence(&self) -> WeightedEvidence {
        match self.certificate_evaluation {
            CertificateEvaluation::Verified(_) => WeightedEvidence::ProvenOptimal,
            _ => WeightedEvidence::IndependentlyVerifiedFeasibleAndAccounted,
        }
    }
    pub fn objective(&self) -> WeightedObjective {
        WeightedObjective::MaximizeAssignedThenMinimizeSuppliedCost
    }
    pub fn certificate_evaluation(&self) -> &CertificateEvaluation {
        &self.certificate_evaluation
    }
}
