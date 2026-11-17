use std::collections::HashSet;

use crate::{
    AssignmentChange, AssignmentChangeReason, Problem, Solution, SolutionComparison,
    SolutionComparisonError, verify_solution,
};

/// Compares two independently verified plans without rerunning either search.
///
/// Models must share the same nonblank, unique variable names in the same order.
/// Names match exactly; names that only differ by surrounding whitespace are
/// considered ambiguous identities. Callers must preserve the meaning of numeric
/// domain values across revisions. The comparison does not verify optimality or
/// search effort, or infer causes when both values remain eligible.
pub fn compare_solutions(
    previous_problem: &Problem,
    previous_solution: &Solution,
    updated_problem: &Problem,
    updated_solution: &Solution,
) -> Result<SolutionComparison, SolutionComparisonError> {
    verify_variable_alignment(previous_problem, updated_problem)?;
    verify_solution(previous_problem, previous_solution)
        .map_err(SolutionComparisonError::InvalidPreviousSolution)?;
    verify_solution(updated_problem, updated_solution)
        .map_err(SolutionComparisonError::InvalidUpdatedSolution)?;

    let mut changes = Vec::new();
    for (index, (&previous_value, &updated_value)) in previous_solution
        .values
        .iter()
        .zip(&updated_solution.values)
        .enumerate()
    {
        if previous_value == updated_value {
            continue;
        }
        let reason = if updated_problem.variables[index]
            .domain
            .contains(&previous_value)
        {
            AssignmentChangeReason::ReasonNotEstablished
        } else {
            AssignmentChangeReason::PreviousValueOutsideUpdatedDomain
        };
        changes.push(AssignmentChange {
            variable_index: index,
            previous_value,
            updated_value,
            reason,
        });
    }

    Ok(SolutionComparison {
        previous_score: previous_solution.score,
        updated_score: updated_solution.score,
        changes,
    })
}

fn verify_variable_alignment(
    previous: &Problem,
    updated: &Problem,
) -> Result<(), SolutionComparisonError> {
    if previous.variables.len() != updated.variables.len() {
        return Err(SolutionComparisonError::VariableCountMismatch);
    }

    let mut unique = HashSet::with_capacity(previous.variables.len());
    for (index, (previous, updated)) in previous
        .variables
        .iter()
        .zip(&updated.variables)
        .enumerate()
    {
        let identity = previous.name.trim();
        if identity.is_empty() || !unique.insert(identity) {
            return Err(SolutionComparisonError::InvalidVariableIdentity(index));
        }
        if previous.name != updated.name {
            return Err(SolutionComparisonError::VariableIdentityMismatch(index));
        }
    }

    Ok(())
}
