use orchestrata_solver::{Constraint, Error, PreparedProblem, Problem, Variable, solve};

fn problem(domain: Vec<i64>, constraints: Vec<Constraint>) -> Problem {
    Problem {
        variables: vec![Variable {
            name: "job".into(),
            domain,
        }],
        constraints,
    }
}

#[test]
fn uses_budget_on_distinct_assignments_and_retains_tie_order() {
    let input = problem(vec![10, -1, 10, -1, 3, 10], vec![]);
    let prepared = PreparedProblem::prepare(input).unwrap();
    let result = prepared.solve(3).unwrap().unwrap();
    assert_eq!(result.values, vec![10]);
    assert_eq!(result.evaluated, 3);
    assert!(matches!(prepared.solve(2), Err(Error::TooManyAssignments)));
}

#[test]
fn rejects_invalid_input_before_search_and_does_not_add_options() {
    assert!(matches!(
        PreparedProblem::prepare(problem(vec![], vec![])),
        Err(Error::EmptyDomain(0))
    ));
    let prepared = PreparedProblem::prepare(problem(
        vec![2, 2],
        vec![Constraint::PreferValue {
            variable: 0,
            value: 7,
            soft_weight: 1,
        }],
    ))
    .unwrap();
    assert_eq!(prepared.solve(1).unwrap().unwrap().values, vec![2]);
}

#[test]
fn unique_domains_match_the_existing_reference_search() {
    let input = problem(
        vec![10, -1, 3],
        vec![Constraint::PreferValue {
            variable: 0,
            value: -1,
            soft_weight: 1,
        }],
    );
    let direct = solve(&input, 3).unwrap().unwrap();
    let result = PreparedProblem::prepare(input).unwrap().solve(3).unwrap().unwrap();
    assert_eq!(result.values, direct.values);
    assert_eq!(result.score, direct.score);
    assert_eq!(result.evaluated, direct.evaluated);
}

#[test]
fn normalizes_each_domain_without_reordering_variables() {
    let input = Problem {
        variables: vec![
            Variable { name: "first".into(), domain: vec![7, 1, 7] },
            Variable { name: "second".into(), domain: vec![4, -2, 6, -2] },
        ],
        constraints: vec![],
    };
    let result = PreparedProblem::prepare(input).unwrap().solve(6).unwrap().unwrap();
    assert_eq!(result.values, vec![7, 4]);
    assert_eq!(result.evaluated, 6);
}
