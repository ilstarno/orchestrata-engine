use crate::module::search_scoring::candidate_penalty;
use crate::{Error, Problem, Score, Solution, validate, verify_solution};

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

    let mut values = vec![0; problem.variables.len()];
    let mut positions = vec![0_usize; problem.variables.len()];
    let mut best: Option<(Vec<i64>, i128)> = None;
    let mut evaluated = 0_u64;

    loop {
        for (index, variable) in problem.variables.iter().enumerate() {
            values[index] = variable.domain[positions[index]];
        }

        evaluated += 1;
        if let Some(penalty) = candidate_penalty(problem, &values)
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
