use orchestrata_solver::capacity_matching::{CapacityMatchProblem, match_capacity};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let workers = ["Ana", "Rami"];
    let jobs = ["Safety inspection", "Lift control check", "Boiler repair"];
    let request = CapacityMatchProblem {
        worker_capacities: vec![1, 1],
        eligible_workers_by_job: vec![vec![0, 1], vec![0], vec![]],
    };

    let plan = match_capacity(&request)?;
    println!("Assigned {} of {} jobs.", plan.assigned, jobs.len());
    for (job, worker) in jobs.iter().zip(&plan.workers_by_job) {
        match worker {
            Some(index) => println!("{job}: {}", workers[*index]),
            None => println!("{job}: unassigned"),
        }
    }
    Ok(())
}
