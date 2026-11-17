use std::ops::ControlFlow;

use crate::{
    CompiledProblem, Error, IncrementalScorer, Problem, SearchOutcome, SearchStop, Solution,
    verify_solution,
};

/// Runs deterministic first-improvement search from the first value of each domain.
///
/// An evaluation is the initial complete assignment or a non-no-op replacement
/// trial. Each accepted move restarts the scan in variable and domain order.
/// Search uses hard score before soft score and never accepts a tie. A limit or
/// exhausted single-variable neighborhood is not a proof about the full space.
pub fn search_locally(problem: &Problem, max_evaluations: u64) -> Result<SearchOutcome, Error> {
    let compiled = CompiledProblem::new(problem)?;
    if max_evaluations == 0 {
        return Ok(SearchOutcome::NoFeasibleIncumbent {
            evaluated: 0,
            stop: SearchStop::EvaluationLimit,
        });
    }

    let initial: Vec<_> = problem
        .variables
        .iter()
        .map(|variable| variable.domain[0])
        .collect();
    let mut scorer = IncrementalScorer::new(&compiled, &initial)?;
    let mut evaluated = 1;

    let stop = loop {
        if let ControlFlow::Break(stop) =
            step(problem, &mut scorer, &mut evaluated, max_evaluations)?
        {
            break stop;
        }
    };

    if !scorer.is_feasible() {
        return Ok(SearchOutcome::NoFeasibleIncumbent { evaluated, stop });
    }

    let solution = Solution {
        values: scorer.values().to_vec(),
        score: scorer.score()?,
        evaluated,
    };
    verify_solution(problem, &solution).map_err(|_| Error::InvalidSolution)?;
    Ok(SearchOutcome::FeasibleIncumbent { solution, stop })
}

fn step(
    problem: &Problem,
    scorer: &mut IncrementalScorer<'_>,
    evaluated: &mut u64,
    max_evaluations: u64,
) -> Result<ControlFlow<SearchStop>, Error> {
    let current_grade = scorer.total;
    for (index, variable) in problem.variables.iter().enumerate() {
        let current_value = scorer.values()[index];
        for &candidate in &variable.domain {
            if candidate == current_value {
                continue;
            }
            if *evaluated == max_evaluations {
                return Ok(ControlFlow::Break(SearchStop::EvaluationLimit));
            }

            let trial = scorer.try_move(index, candidate)?;
            *evaluated += 1;
            if trial.wide_score() > current_grade {
                trial.commit();
                return Ok(ControlFlow::Continue(()));
            }
        }
    }

    Ok(ControlFlow::Break(SearchStop::NeighborhoodExhausted))
}
