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
    search(problem, max_evaluations, false)
}

/// Opt into deterministic single-variable replacements followed by pair swaps.
///
/// Each pass tries the same replacements as `search_locally`. If none improve,
/// it tries unordered variable pairs in index order. Equal-value exchanges and
/// swaps outside either receiving domain are not candidates. Each scored swap
/// uses one evaluation and improvements restart the pass. Exhaustion is local
/// to these two neighborhoods; it never proves optimality or infeasibility.
/// The budget counts scored assignments, not pair compatibility checks.
pub fn search_locally_with_swaps(
    problem: &Problem,
    max_evaluations: u64,
) -> Result<SearchOutcome, Error> {
    search(problem, max_evaluations, true)
}

fn search(
    problem: &Problem,
    max_evaluations: u64,
    enable_swaps: bool,
) -> Result<SearchOutcome, Error> {
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
    let mut swap_variables = Vec::new();
    if enable_swaps {
        // A singleton domain cannot receive a distinct value.
        swap_variables.extend(problem.variables.iter().enumerate().filter_map(
            |(index, variable)| {
                variable.domain[1..]
                    .iter()
                    .any(|&value| value != variable.domain[0])
                    .then_some(index)
            },
        ));
    }

    let stop = loop {
        if let ControlFlow::Break(stop) = step(
            problem,
            &mut scorer,
            &mut evaluated,
            max_evaluations,
            &swap_variables,
        )? {
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
    swap_variables: &[usize],
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

    for (position, &left) in swap_variables.iter().enumerate() {
        for &right in &swap_variables[position + 1..] {
            let a = scorer.values()[left];
            let b = scorer.values()[right];
            if a == b
                || !problem.variables[left].domain.contains(&b)
                || !problem.variables[right].domain.contains(&a)
            {
                continue;
            }
            if *evaluated == max_evaluations {
                return Ok(ControlFlow::Break(SearchStop::EvaluationLimit));
            }
            let improved = scorer.improve_swap(left, right)?;
            *evaluated += 1;
            if improved {
                return Ok(ControlFlow::Continue(()));
            }
        }
    }

    Ok(ControlFlow::Break(SearchStop::NeighborhoodExhausted))
}
