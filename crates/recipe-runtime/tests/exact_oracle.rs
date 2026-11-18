use orchestrata_recipe_runtime::allocate_named;
use serde_json::{Value, json};
const XML: &str = include_str!("fixtures/maintenance-allocation.xml");

// Exhaustive enumeration is independent of the solver's augmenting-path algorithm.
fn optimum(data: &Value) -> usize {
    let mut best = 0;
    for encoding in 0..64 {
        let mut n = encoding;
        let mut counts = [0; 3];
        let mut assigned = 0;
        let mut valid = true;
        for job in 0..3 {
            let choice = n % 4;
            n /= 4;
            if choice == 3 {
                continue;
            }
            let worker = &data["technicians"][choice];
            valid &= worker["available"] == true
                && worker["skills"]
                    .as_array()
                    .unwrap()
                    .contains(&data["work-orders"][job]["required_skill"]);
            counts[choice] += 1;
            assigned += 1;
        }
        valid &= counts
            .iter()
            .enumerate()
            .all(|(w, &count)| count <= data["technicians"][w]["capacity"].as_u64().unwrap());
        if valid {
            best = best.max(assigned);
        }
    }
    best
}

#[test]
fn all_three_by_three_eligibility_graphs_and_availability_masks_match_exact_optima() {
    for graph in 0..512 {
        for availability in 0..8 {
            let data = json!({
                "work-orders": (0..3).map(|j| json!({"id":format!("job-{j}"),
                    "name":"Synthetic job", "required_skill":format!("skill-{j}")})).collect::<Vec<_>>(),
                "technicians": (0..3).map(|w| json!({"id":format!("worker-{w}"),
                    "name":"Synthetic worker", "available": availability & (1 << w) != 0,
                    "capacity": (graph + w) % 3,
                    "skills":(0..3).filter(|j| graph & (1 << (j * 3 + w)) != 0)
                        .map(|j|format!("skill-{j}")).collect::<Vec<_>>() })).collect::<Vec<_>>()
            });
            let result = allocate_named(XML, &data.to_string()).unwrap();
            assert_eq!(
                result.assigned,
                optimum(&data),
                "graph={graph}, availability={availability}"
            );
            assert_eq!(result.allocations.len(), 3);
            let mut counts = [0; 3];
            for (j, allocation) in result.allocations.iter().enumerate() {
                assert_eq!(allocation.job_id, format!("job-{j}"));
                if let Some(id) = &allocation.worker_id {
                    let w = id
                        .strip_prefix("worker-")
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    assert!(w < 3);
                    assert_eq!(data["technicians"][w]["available"], true);
                    assert!(
                        data["technicians"][w]["skills"]
                            .as_array()
                            .unwrap()
                            .contains(&data["work-orders"][j]["required_skill"])
                    );
                    counts[w] += 1;
                    assert!(allocation.unassigned_reason.is_none());
                } else {
                    assert!(allocation.unassigned_reason.is_some());
                }
            }
            assert_eq!(counts.iter().sum::<u64>() as usize, result.assigned);
            for (w, count) in counts.iter().enumerate() {
                assert!(*count <= data["technicians"][w]["capacity"].as_u64().unwrap());
            }
        }
    }
}
