use crate::capacity_matching::{
    CapacityMatchError, CapacityMatchProblem, match_capacity,
    weighted::{
        WeightedMatchAssignment, WeightedMatchProblem, WorkerCost, match_capacity_min_cost,
    },
};

fn edge(worker: usize, cost: u32) -> WorkerCost {
    WorkerCost { worker, cost }
}

fn assert_valid(problem: &WeightedMatchProblem, plan: &WeightedMatchAssignment) {
    let mut used = vec![0; problem.worker_capacities.len()];
    let mut assigned = 0;
    let mut total = 0;
    assert_eq!(
        plan.workers_by_job.len(),
        problem.eligible_workers_by_job.len()
    );
    for (choices, worker) in problem
        .eligible_workers_by_job
        .iter()
        .zip(&plan.workers_by_job)
    {
        if let Some(worker) = *worker {
            used[worker] += 1;
            assigned += 1;
            total += u64::from(choices.iter().find(|e| e.worker == worker).unwrap().cost);
        }
    }
    assert!(
        used.iter()
            .zip(&problem.worker_capacities)
            .all(|(n, cap)| n <= cap)
    );
    assert_eq!((assigned, total), (plan.assigned, plan.total_cost));
}

fn brute_force(choices: &[Vec<WorkerCost>], remaining: &mut [usize]) -> (usize, u64) {
    let Some((first, rest)) = choices.split_first() else {
        return (0, 0);
    };
    let mut best = brute_force(rest, remaining);
    for edge in first {
        if remaining[edge.worker] == 0 {
            continue;
        }
        remaining[edge.worker] -= 1;
        let (count, cost) = brute_force(rest, remaining);
        remaining[edge.worker] += 1;
        let candidate = (count + 1, cost + u64::from(edge.cost));
        if candidate.0 > best.0 || (candidate.0 == best.0 && candidate.1 < best.1) {
            best = candidate;
        }
    }
    best
}

#[test]
fn extra_placement_wins_even_at_maximum_cost() {
    let problem = WeightedMatchProblem {
        worker_capacities: vec![1, 1],
        eligible_workers_by_job: vec![
            vec![edge(0, 0), edge(1, u32::MAX)],
            vec![edge(0, 0)],
            vec![],
        ],
    };
    let plan = match_capacity_min_cost(&problem).unwrap();
    assert_eq!(plan.workers_by_job, vec![Some(1), Some(0), None]);
    assert_eq!(plan.assigned, 2);
    assert_eq!(plan.total_cost, u64::from(u32::MAX));
    assert_valid(&problem, &plan);
}

#[test]
fn a_greedy_cheap_first_edge_can_lose_the_lowest_total_cost() {
    let problem = WeightedMatchProblem {
        worker_capacities: vec![1, 1],
        eligible_workers_by_job: vec![vec![edge(0, 0), edge(1, 2)], vec![edge(0, 1), edge(1, 100)]],
    };
    let plan = match_capacity_min_cost(&problem).unwrap();
    assert_eq!(plan.workers_by_job, vec![Some(1), Some(0)]);
    assert_eq!((plan.assigned, plan.total_cost), (2, 3));
    assert_valid(&problem, &plan);
}

#[test]
fn exhaustive_weighted_graphs_agree_with_independent_oracle() {
    for mask in 0..512usize {
        for capacity_code in 0..27usize {
            for variant in 0..4usize {
                let problem = WeightedMatchProblem {
                    worker_capacities: vec![
                        capacity_code % 3,
                        capacity_code / 3 % 3,
                        capacity_code / 9,
                    ],
                    eligible_workers_by_job: (0..3)
                        .map(|j| {
                            (0..3)
                                .filter(|&w| mask & (1 << (j * 3 + w)) != 0)
                                .map(|w| {
                                    edge(
                                        w,
                                        match variant {
                                            0 => 0,
                                            1 => (j * w + w) as u32,
                                            2 => ((3 - j) * (3 - w)) as u32,
                                            _ => u32::MAX - (j * 3 + w) as u32,
                                        },
                                    )
                                })
                                .collect()
                        })
                        .collect(),
                };
                let expected = brute_force(
                    &problem.eligible_workers_by_job,
                    &mut problem.worker_capacities.clone(),
                );
                let plan = match_capacity_min_cost(&problem).unwrap();
                assert_eq!(
                    (plan.assigned, plan.total_cost),
                    expected,
                    "mask={mask} capacity={capacity_code} variant={variant}"
                );
                assert_valid(&problem, &plan);
                if variant == 0 {
                    let unweighted = CapacityMatchProblem {
                        worker_capacities: problem.worker_capacities.clone(),
                        eligible_workers_by_job: problem
                            .eligible_workers_by_job
                            .iter()
                            .map(|c| c.iter().map(|e| e.worker).collect())
                            .collect(),
                    };
                    assert_eq!(plan.assigned, match_capacity(&unweighted).unwrap().assigned);
                }
            }
        }
    }
}

#[test]
fn dense_limits_and_large_cost_sums_are_exact() {
    let n = 200;
    let problem = WeightedMatchProblem {
        worker_capacities: vec![1; n],
        eligible_workers_by_job: (0..n)
            .map(|job| {
                (0..n)
                    .map(|worker| {
                        edge(
                            worker,
                            if job == worker {
                                u32::MAX - 1
                            } else {
                                u32::MAX
                            },
                        )
                    })
                    .collect()
            })
            .collect(),
    };
    let plan = match_capacity_min_cost(&problem).unwrap();
    assert_eq!(plan.assigned, n);
    assert_eq!(plan.total_cost, (n as u64) * u64::from(u32::MAX - 1));
    assert_eq!(plan.workers_by_job, (0..n).map(Some).collect::<Vec<_>>());
    assert_valid(&problem, &plan);
    assert_eq!(plan, match_capacity_min_cost(&problem).unwrap());
}

#[test]
fn validates_same_hard_boundaries_without_implicit_defaults() {
    let mut problem = WeightedMatchProblem {
        worker_capacities: vec![],
        eligible_workers_by_job: vec![vec![]],
    };
    assert_eq!(
        match_capacity_min_cost(&problem).unwrap().workers_by_job,
        vec![None]
    );
    problem.worker_capacities = vec![1];
    problem.eligible_workers_by_job = vec![vec![edge(0, 1), edge(0, 2)]];
    assert_eq!(
        match_capacity_min_cost(&problem),
        Err(CapacityMatchError::RepeatedWorker { job: 0, worker: 0 })
    );
    problem.eligible_workers_by_job = vec![vec![edge(1, 0)]];
    assert_eq!(
        match_capacity_min_cost(&problem),
        Err(CapacityMatchError::UnknownWorker { job: 0, worker: 1 })
    );
    problem.worker_capacities = vec![201];
    assert_eq!(
        match_capacity_min_cost(&problem),
        Err(CapacityMatchError::InvalidCapacity { worker: 0 })
    );
    problem.worker_capacities = vec![0; 201];
    assert_eq!(
        match_capacity_min_cost(&problem),
        Err(CapacityMatchError::TooManyRecords)
    );
    problem.worker_capacities = vec![];
    problem.eligible_workers_by_job = vec![vec![]; 201];
    assert_eq!(
        match_capacity_min_cost(&problem),
        Err(CapacityMatchError::TooManyRecords)
    );
}
