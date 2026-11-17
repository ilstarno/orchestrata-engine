use super::{CapacityMatchError, CapacityMatchProblem, match_capacity};

fn verify(problem: &CapacityMatchProblem) -> usize {
    let result = match_capacity(problem).expect("valid data");
    assert_eq!(result, match_capacity(problem).expect("repeat"));
    assert_eq!(
        result.workers_by_job.len(),
        problem.eligible_workers_by_job.len()
    );
    let mut loads = vec![0; problem.worker_capacities.len()];
    for (job, worker) in result.workers_by_job.into_iter().enumerate() {
        if let Some(worker) = worker {
            assert!(problem.eligible_workers_by_job[job].contains(&worker));
            loads[worker] += 1;
        }
    }
    assert!(
        loads
            .iter()
            .zip(&problem.worker_capacities)
            .all(|(load, cap)| load <= cap)
    );
    assert_eq!(result.assigned, loads.iter().sum::<usize>());
    result.assigned
}

fn oracle(eligible: &[Vec<usize>], remaining: &mut [usize]) -> usize {
    let Some((options, rest)) = eligible.split_first() else {
        return 0;
    };
    let mut best = oracle(rest, remaining);
    for &worker in options {
        if remaining[worker] > 0 {
            remaining[worker] -= 1;
            best = best.max(1 + oracle(rest, remaining));
            remaining[worker] += 1;
        }
    }
    best
}

#[test]
fn all_tiny_cases_match_enumeration() {
    for mask in 0..(1 << 12) {
        let eligibility = (0..4)
            .map(|job| {
                (0..3)
                    .filter(|worker| mask & (1 << (job * 3 + worker)) != 0)
                    .collect()
            })
            .collect::<Vec<_>>();
        for caps in [[1, 1, 1], [2, 0, 1], [0, 2, 2], [3, 1, 0]] {
            let problem = CapacityMatchProblem {
                worker_capacities: caps.into(),
                eligible_workers_by_job: eligibility.clone(),
            };
            assert_eq!(
                verify(&problem),
                oracle(&eligibility, &mut { caps }),
                "{problem:?}"
            );
        }
    }
}

#[test]
fn opens_a_slot_by_moving_a_job() {
    let problem = CapacityMatchProblem {
        worker_capacities: vec![2, 1],
        eligible_workers_by_job: vec![vec![0, 1], vec![0], vec![0], vec![]],
    };
    assert_eq!(verify(&problem), 3);
    assert_eq!(
        match_capacity(&problem).unwrap().workers_by_job,
        vec![Some(1), Some(0), Some(0), None]
    );
}

#[test]
fn dense_two_hundred_job_case() {
    let problem = CapacityMatchProblem {
        worker_capacities: vec![1; 200],
        eligible_workers_by_job: vec![(0..200).collect(); 200],
    };
    assert_eq!(verify(&problem), 200);
}

#[test]
fn rejects_invalid_indices_repeats_and_limits() {
    let mut problem = CapacityMatchProblem {
        worker_capacities: vec![1],
        eligible_workers_by_job: vec![vec![2]],
    };
    assert_eq!(
        match_capacity(&problem),
        Err(CapacityMatchError::UnknownWorker { job: 0, worker: 2 })
    );
    problem.eligible_workers_by_job[0] = vec![0, 0];
    assert_eq!(
        match_capacity(&problem),
        Err(CapacityMatchError::RepeatedWorker { job: 0, worker: 0 })
    );
    problem.worker_capacities[0] = 201;
    assert_eq!(
        match_capacity(&problem),
        Err(CapacityMatchError::InvalidCapacity { worker: 0 })
    );
    problem.eligible_workers_by_job = vec![vec![]; 201];
    assert_eq!(
        match_capacity(&problem),
        Err(CapacityMatchError::TooManyRecords)
    );
}
