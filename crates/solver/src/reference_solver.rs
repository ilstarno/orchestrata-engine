use super::{candidate_scratch::CandidateScratch, exact_assignments::ExactAssignments};
use crate::module::search_scoring::candidate_penalty;
use crate::{Error, Problem, Score, Solution, validate, verify_solution};

/// Finds the best hard-feasible assignment in declared variable and domain order.
///
/// Returns `None` only after exhaustive search proves infeasibility. If the full
/// search space exceeds the evaluation limit, no assignments are evaluated.
pub fn solve(problem: &Problem, max_evaluations: u64) -> Result<Option<Solution>, Error> {
    validate(problem)?;
    solve_validated(problem, max_evaluations)
}

pub(crate) fn solve_validated(
    problem: &Problem,
    max_evaluations: u64,
) -> Result<Option<Solution>, Error> {
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

    let mut assignments = ExactAssignments::new(&problem.variables);
    let mut scratch = CandidateScratch::default();
    let mut best: Option<(Vec<i64>, i128)> = None;
    let mut evaluated = 0_u64;

    loop {
        let values = assignments.values();
        evaluated += 1;
        if let Some(penalty) = candidate_penalty(problem, values, &mut scratch)
            && best.as_ref().is_none_or(|(_, best)| penalty < *best)
        {
            best = Some((values.to_vec(), penalty));
        }

        if !assignments.advance() {
            break;
        }
    }

    let Some((values, penalty)) = best else {
        return Ok(None);
    };

    let soft = i64::try_from(-penalty).map_err(|_| Error::ScoreOverflow)?;
    let solution = Solution {
        values,
        score: Score { hard: 0, soft },
        evaluated,
    };
    verify_solution(problem, &solution).map_err(|_| Error::InvalidSolution)?;

    Ok(Some(solution))
}
