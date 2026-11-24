use super::{
    resource_load_totals::ResourceLoadTotals, solution_validation::verify_validated_solution,
};
use crate::module::search_scoring::candidate_penalty;
use crate::{
    Error, Problem, Score, Solution, SolutionEvidence, SolveReport, SolveTermination, validate,
};

/// Finds the best hard-feasible assignment in declared variable and domain order.
///
/// Returns `None` only after exhaustive search proves infeasibility. If the full
/// search space exceeds the evaluation limit, no assignments are evaluated.
pub fn solve(problem: &Problem, max_evaluations: u64) -> Result<Option<Solution>, Error> {
    validate(problem)?;

    let mut combinations = 1_u64;
    for variable in &problem.variables {
        let size = u64::try_from(variable.domain.len()).map_err(|_| Error::TooManyAssignments)?;
        combinations = combinations
            .checked_mul(size)
            .ok_or(Error::TooManyAssignments)?;
    }

    if combinations > max_evaluations {
        return Err(Error::TooManyAssignments);
    }

    let report = enumerate(problem, max_evaluations)?;
    match report.into_evidence() {
        SolutionEvidence::ProvenOptimal(solution) => Ok(Some(solution)),
        SolutionEvidence::ProvenInfeasible => Ok(None),
        _ => unreachable!("preflight admits only exhaustive search"),
    }
}

/// Enumerates at most `max_evaluations` assignments in declared order.
/// A limit can return a separately verified incumbent or unknown, never a proof
/// of optimality/infeasibility. Validation and final checking are outside this
/// evaluation budget; this API provides no wall-clock or cancellation guarantee.
/// Invalid inputs and arithmetic/checking failures remain explicit errors.
pub fn solve_bounded(problem: &Problem, max_evaluations: u64) -> Result<SolveReport, Error> {
    validate(problem)?;
    enumerate(problem, max_evaluations)
}

fn enumerate(problem: &Problem, max_evaluations: u64) -> Result<SolveReport, Error> {
    let mut values = vec![0; problem.variables.len()];
    let mut positions = vec![0_usize; problem.variables.len()];
    let mut best: Option<(Vec<i64>, i128)> = None;
    let mut evaluated = 0_u64;
    let mut resource_loads = ResourceLoadTotals::default();

    let termination = loop {
        if evaluated == max_evaluations {
            break SolveTermination::EvaluationLimit;
        }
        for (index, variable) in problem.variables.iter().enumerate() {
            values[index] = variable.domain[positions[index]];
        }

        evaluated += 1;
        if let Some(penalty) = candidate_penalty(problem, &values, &mut resource_loads)
            && best.as_ref().is_none_or(|(_, best)| penalty < *best)
        {
            best = Some((values.clone(), penalty));
        }

        let mut next = positions.len();
        while next > 0 {
            next -= 1;
            positions[next] += 1;
            if positions[next] < problem.variables[next].domain.len() {
                break;
            }
            positions[next] = 0;
        }

        if positions.is_empty() || (next == 0 && positions[0] == 0) {
            break SolveTermination::SearchExhausted;
        }
    };

    let Some((values, penalty)) = best else {
        let evidence = match termination {
            SolveTermination::SearchExhausted => SolutionEvidence::ProvenInfeasible,
            SolveTermination::EvaluationLimit => SolutionEvidence::Unknown,
        };
        return Ok(SolveReport::new(termination, evidence, evaluated));
    };

    let soft = i64::try_from(-penalty).map_err(|_| Error::ScoreOverflow)?;
    let solution = Solution {
        values,
        score: Score { hard: 0, soft },
        evaluated,
    };
    verify_validated_solution(problem, &solution).map_err(|_| Error::InvalidSolution)?;

    let evidence = match termination {
        SolveTermination::SearchExhausted => SolutionEvidence::ProvenOptimal(solution),
        SolveTermination::EvaluationLimit => SolutionEvidence::VerifiedFeasible(solution),
    };
    Ok(SolveReport::new(termination, evidence, evaluated))
}
