use super::{
    CertificateEvaluation, WeightedMatchAssignment, WeightedMatchProblem, WeightedReport,
    WeightedTermination, WeightedVerificationError, match_capacity_min_cost,
    verify_weighted_assignment, verify_weighted_feasibility,
};

/// Runs the existing solver and independently checks feasibility/accounting.
/// Optional verification work counts residual arc examinations only;
/// it does not limit solving, validation, allocation or elapsed time. Only a
/// successful independent certificate promotes the evidence to proven optimal.
pub fn solve_weighted_report(
    problem: &WeightedMatchProblem,
    verification_work: Option<u64>,
) -> Result<WeightedReport, WeightedVerificationError> {
    let assignment =
        match_capacity_min_cost(problem).map_err(WeightedVerificationError::InvalidProblem)?;
    report(
        problem,
        assignment,
        WeightedTermination::Completed,
        verification_work,
    )
}

/// Checks a supplied assignment without claiming that a solver ran.
/// Invalid feasibility/accounting is an error, never a feasible report.
pub fn assess_weighted_assignment(
    problem: &WeightedMatchProblem,
    assignment: WeightedMatchAssignment,
    verification_work: Option<u64>,
) -> Result<WeightedReport, WeightedVerificationError> {
    report(
        problem,
        assignment,
        WeightedTermination::NotRun,
        verification_work,
    )
}

fn report(
    problem: &WeightedMatchProblem,
    assignment: WeightedMatchAssignment,
    termination: WeightedTermination,
    verification_work: Option<u64>,
) -> Result<WeightedReport, WeightedVerificationError> {
    verify_weighted_feasibility(problem, &assignment)?;
    let certificate_evaluation = match verification_work {
        None => CertificateEvaluation::NotRequested,
        Some(work) => match verify_weighted_assignment(problem, &assignment, work) {
            Ok(check) => CertificateEvaluation::Verified(check),
            Err(WeightedVerificationError::WorkLimit) => CertificateEvaluation::WorkLimit,
            Err(error) => CertificateEvaluation::Rejected(error),
        },
    };
    Ok(WeightedReport::new(
        assignment,
        termination,
        certificate_evaluation,
    ))
}
