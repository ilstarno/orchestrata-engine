use crate::capacity_matching::{
    CapacityMatchError,
    weighted::{
        WeightedMatchProblem, WorkerCost, match_capacity_min_cost,
        penalized::{PenaltyJob, PenaltyMatchProblem, match_capacity_with_penalties},
    },
};

fn edge(worker: usize, cost: u32) -> WorkerCost {
    WorkerCost { worker, cost }
}
fn oracle(jobs: &[PenaltyJob], caps: &mut [usize]) -> u64 {
    let Some((first, rest)) = jobs.split_first() else {
        return 0;
    };
    let mut best = u64::from(first.unassigned_cost) + oracle(rest, caps);
    for e in &first.eligible_workers {
        if caps[e.worker] > 0 {
            caps[e.worker] -= 1;
            best = best.min(u64::from(e.cost) + oracle(rest, caps));
            caps[e.worker] += 1;
        }
    }
    best
}
fn verify(problem: &PenaltyMatchProblem) {
    let result = match_capacity_with_penalties(problem).unwrap();
    assert_eq!(
        result.total_cost,
        oracle(&problem.jobs, &mut problem.worker_capacities.clone())
    );
    let mut used = vec![0; problem.worker_capacities.len()];
    let (mut service, mut missed, mut assigned) = (0, 0, 0);
    for (job, worker) in problem.jobs.iter().zip(&result.workers_by_job) {
        match worker {
            Some(w) => {
                used[*w] += 1;
                assigned += 1;
                service += u64::from(
                    job.eligible_workers
                        .iter()
                        .find(|e| e.worker == *w)
                        .unwrap()
                        .cost,
                );
            }
            None => missed += u64::from(job.unassigned_cost),
        }
    }
    assert!(
        used.iter()
            .zip(&problem.worker_capacities)
            .all(|(u, c)| u <= c)
    );
    assert_eq!(
        (service, missed, assigned),
        (result.service_cost, result.unassigned_cost, result.assigned)
    );
    assert_eq!(service + missed, result.total_cost);
    assert_eq!(result, match_capacity_with_penalties(problem).unwrap());
}

#[test]
fn exhaustive_52488_small_problems_match_independent_enumeration() {
    for encoded in 0..729 {
        for cap_a in 0..3 {
            for cap_b in 0..3 {
                for penalties in 0..8 {
                    let mut code = encoded;
                    let jobs = (0..3)
                        .map(|job| {
                            let eligible_workers = (0..2)
                                .filter_map(|worker| {
                                    let digit = code % 3;
                                    code /= 3;
                                    (digit != 0)
                                        .then_some(edge(worker, if digit == 1 { 0 } else { 3 }))
                                })
                                .collect();
                            PenaltyJob {
                                eligible_workers,
                                unassigned_cost: if penalties & (1 << job) == 0 { 0 } else { 2 },
                            }
                        })
                        .collect();
                    verify(&PenaltyMatchProblem {
                        worker_capacities: vec![cap_a, cap_b],
                        jobs,
                    });
                }
            }
        }
    }
}

#[test]
fn contact_center_batch_changes_coverage_and_priority() {
    // Fraud, billing, routine, language support, outage, compliance; all costs cents.
    let p = PenaltyMatchProblem {
        worker_capacities: vec![2, 1, 0, 1],
        jobs: vec![
            PenaltyJob {
                eligible_workers: vec![edge(0, 100)],
                unassigned_cost: 10000,
            },
            PenaltyJob {
                eligible_workers: vec![edge(0, 200), edge(1, 300)],
                unassigned_cost: 5000,
            },
            PenaltyJob {
                eligible_workers: vec![edge(3, 900)],
                unassigned_cost: 100,
            },
            PenaltyJob {
                eligible_workers: vec![edge(2, 100)],
                unassigned_cost: 8000,
            },
            PenaltyJob {
                eligible_workers: vec![edge(0, 400)],
                unassigned_cost: 6000,
            },
            PenaltyJob {
                eligible_workers: vec![edge(1, 100)],
                unassigned_cost: 7000,
            },
        ],
    };
    verify(&p);
    let result = match_capacity_with_penalties(&p).unwrap();
    assert_eq!(
        result.workers_by_job,
        vec![Some(0), None, None, None, Some(0), Some(1)]
    );
    assert_eq!(
        (
            result.assigned,
            result.service_cost,
            result.unassigned_cost,
            result.total_cost
        ),
        (3, 600, 13100, 13700)
    );
    let old = match_capacity_min_cost(&WeightedMatchProblem {
        worker_capacities: p.worker_capacities.clone(),
        eligible_workers_by_job: p.jobs.iter().map(|j| j.eligible_workers.clone()).collect(),
    })
    .unwrap();
    assert_eq!(old.assigned, 4);
    let mut changed = p.clone();
    changed.jobs[1].unassigned_cost = 9000;
    verify(&changed);
    let next = match_capacity_with_penalties(&changed).unwrap();
    assert_eq!(next.workers_by_job[1], Some(0));
    assert_eq!(next.workers_by_job[4], None);
    let mut unavailable = p;
    unavailable.worker_capacities.fill(0);
    verify(&unavailable);
    assert_eq!(
        match_capacity_with_penalties(&unavailable)
            .unwrap()
            .assigned,
        0
    );
}

#[test]
fn reroutes_instead_of_greedily_taking_the_cheapest_first_edge() {
    let p = PenaltyMatchProblem {
        worker_capacities: vec![1, 1],
        jobs: vec![
            PenaltyJob {
                eligible_workers: vec![edge(0, 0), edge(1, 2)],
                unassigned_cost: 20,
            },
            PenaltyJob {
                eligible_workers: vec![edge(0, 1)],
                unassigned_cost: 100,
            },
        ],
    };
    verify(&p);
    assert_eq!(
        match_capacity_with_penalties(&p).unwrap().workers_by_job,
        vec![Some(1), Some(0)]
    );
}

#[test]
fn bounds_invalid_references_duplicates_and_capacity_are_rejected() {
    let mut p = PenaltyMatchProblem {
        worker_capacities: vec![1],
        jobs: vec![PenaltyJob {
            eligible_workers: vec![edge(1, 0)],
            unassigned_cost: 0,
        }],
    };
    assert!(matches!(
        match_capacity_with_penalties(&p),
        Err(CapacityMatchError::UnknownWorker { .. })
    ));
    p.jobs[0].eligible_workers = vec![edge(0, 0), edge(0, 1)];
    assert!(matches!(
        match_capacity_with_penalties(&p),
        Err(CapacityMatchError::RepeatedWorker { .. })
    ));
    p.jobs[0].eligible_workers.clear();
    p.worker_capacities[0] = 201;
    assert!(matches!(
        match_capacity_with_penalties(&p),
        Err(CapacityMatchError::InvalidCapacity { .. })
    ));
    p.worker_capacities[0] = 1;
    p.jobs = vec![p.jobs[0].clone(); 201];
    assert_eq!(
        match_capacity_with_penalties(&p),
        Err(CapacityMatchError::TooManyRecords)
    );
}

#[test]
fn two_hundred_jobs_large_costs_and_empty_inputs() {
    let p = PenaltyMatchProblem {
        worker_capacities: vec![200],
        jobs: vec![
            PenaltyJob {
                eligible_workers: vec![edge(0, u32::MAX - 1)],
                unassigned_cost: u32::MAX
            };
            200
        ],
    };
    let result = match_capacity_with_penalties(&p).unwrap();
    assert_eq!(result.assigned, 200);
    assert_eq!(result.total_cost, 200 * u64::from(u32::MAX - 1));
    let empty = PenaltyMatchProblem {
        worker_capacities: vec![],
        jobs: vec![],
    };
    verify(&empty);
    let omitted = PenaltyMatchProblem {
        worker_capacities: vec![],
        jobs: vec![
            PenaltyJob {
                eligible_workers: vec![],
                unassigned_cost: u32::MAX
            };
            200
        ],
    };
    assert_eq!(
        match_capacity_with_penalties(&omitted).unwrap().total_cost,
        200 * u64::from(u32::MAX)
    );
}

#[test]
fn dense_two_hundred_by_two_hundred_boundary_replays_exactly() {
    let problem = PenaltyMatchProblem {
        worker_capacities: vec![1; 200],
        jobs: vec![
            PenaltyJob {
                eligible_workers: (0..200).map(|worker| edge(worker, 1)).collect(),
                unassigned_cost: 2,
            };
            200
        ],
    };
    let result = match_capacity_with_penalties(&problem).unwrap();
    // Each served job costs at least one, each omitted job costs two.
    // A perfect matching exists, so 200 is an independent exact lower bound.
    assert_eq!((result.assigned, result.total_cost), (200, 200));
    let mut used = [false; 200];
    for worker in &result.workers_by_job {
        assert!(!std::mem::replace(&mut used[worker.unwrap()], true));
    }
    assert_eq!(result, match_capacity_with_penalties(&problem).unwrap());
}
