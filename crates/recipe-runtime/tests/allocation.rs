use orchestrata_recipe_runtime::{UnassignedReason, allocate_named};
use serde_json::{Value, json};
use std::process::Command;

const XML: &str = include_str!("fixtures/maintenance-allocation.xml");
const INPUT: &str = include_str!("fixtures/maintenance-inputs.json");
fn input() -> Value {
    serde_json::from_str(INPUT).unwrap()
}

#[test]
fn maintenance_ids_and_unavailable_reason_are_exact() {
    let result = allocate_named(XML, INPUT).unwrap();
    assert_eq!(result.assigned, 3);
    assert_eq!(result.output_field, "allocations");
    assert_eq!(
        result
            .allocations
            .iter()
            .map(|a| a.job_id.as_str())
            .collect::<Vec<_>>(),
        ["job-0", "job-1", "job-2", "job-3"]
    );
    assert_eq!(
        result
            .allocations
            .iter()
            .map(|a| a.worker_id.as_deref())
            .collect::<Vec<_>>(),
        [
            Some("repair-tech"),
            Some("inspection-tech"),
            Some("repair-tech"),
            None
        ]
    );
    assert_eq!(
        result.allocations[3].unassigned_reason,
        Some(UnassignedReason::MatchingWorkersUnavailable)
    );
    assert!(
        result.allocations[..3]
            .iter()
            .all(|a| a.unassigned_reason.is_none())
    );
    assert_eq!(result, allocate_named(XML, INPUT).unwrap());
}

#[test]
fn availability_capacity_and_skill_shortages_have_distinct_reasons() {
    let mut data = input();
    data["technicians"][2]["available"] = json!(true);
    let result = allocate_named(XML, &data.to_string()).unwrap();
    assert_eq!(result.assigned, 4);
    assert_eq!(
        result.allocations[3].worker_id.as_deref(),
        Some("electrical-tech")
    );
    data["technicians"][0]["capacity"] = json!(1);
    let result = allocate_named(XML, &data.to_string()).unwrap();
    assert_eq!(result.assigned, 3);
    assert_eq!(
        result
            .allocations
            .iter()
            .find(|a| a.worker_id.is_none())
            .unwrap()
            .unassigned_reason,
        Some(UnassignedReason::CapacityExhausted)
    );
    data["technicians"][0]["capacity"] = json!(0);
    let result = allocate_named(XML, &data.to_string()).unwrap();
    assert_eq!(result.assigned, 2);
    assert_eq!(
        result.allocations[0].unassigned_reason,
        Some(UnassignedReason::NoAvailableCapacity)
    );
    data["work-orders"][0]["required_skill"] = json!("Roofing");
    let result = allocate_named(XML, &data.to_string()).unwrap();
    assert_eq!(
        result.allocations[0].unassigned_reason,
        Some(UnassignedReason::NoMatchingSkill)
    );
}

#[test]
fn ids_survive_worker_reordering_and_duplicate_display_names() {
    let mut data = input();
    data["technicians"].as_array_mut().unwrap().reverse();
    assert_eq!(
        allocate_named(XML, INPUT).unwrap(),
        allocate_named(XML, &data.to_string()).unwrap()
    );
}

#[test]
fn malformed_records_and_unsupported_recipes_fail_before_a_result() {
    for field in ["work-orders", "technicians"] {
        let mut data = input();
        data[field][0] = json!(["job-0", "Synthetic job", "Repair"]);
        assert!(allocate_named(XML, &data.to_string()).is_err());
        let mut data = input();
        data[field][0]["unexpected"] = json!(true);
        assert!(allocate_named(XML, &data.to_string()).is_err());
        let mut data = input();
        data[field][1]["id"] = data[field][0]["id"].clone();
        assert!(allocate_named(XML, &data.to_string()).is_err());
    }
    assert!(
        allocate_named(
            XML,
            r#"{"work-orders":[],"work-orders":[],"technicians":[]}"#
        )
        .is_err()
    );
    assert!(allocate_named(&XML.replace("maximum-coverage", "minimum-cost"), INPUT).is_err());
    assert!(allocate_named(include_str!("fixtures/recipes/field-service.xml"), INPUT).is_err());
    assert!(
        allocate_named(
            include_str!("fixtures/recipes/executable/field-service/recipe.xml"),
            INPUT
        )
        .is_err()
    );
}

#[test]
fn changing_the_xml_changes_execution() {
    let mut data = input();
    data["work-orders"][0]["required_skill"] = json!("repair");
    assert_eq!(allocate_named(XML, &data.to_string()).unwrap().assigned, 2);
    let insensitive = XML.replace("match=\"exact\"", "match=\"ascii-case-insensitive\"");
    assert_eq!(
        allocate_named(&insensitive, &data.to_string())
            .unwrap()
            .assigned,
        3
    );
    let renamed = XML.replace("work-orders", "jobs");
    assert!(allocate_named(&renamed, &data.to_string()).is_err());
    let jobs = data.as_object_mut().unwrap().remove("work-orders").unwrap();
    data["jobs"] = jobs;
    assert_eq!(
        allocate_named(&renamed, &data.to_string())
            .unwrap()
            .assigned,
        2
    );
}

#[test]
fn cli_solves_xml_with_stable_ids() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new(env!("CARGO_BIN_EXE_allocate-recipe"))
        .arg(root.join("tests/fixtures/maintenance-allocation.xml"))
        .arg(root.join("tests/fixtures/maintenance-inputs.json"))
        .output()
        .unwrap();
    assert!(output.status.success());
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["assigned"], 3);
    assert_eq!(result["allocations"][3]["job_id"], "job-3");
    assert_eq!(
        result["allocations"][3]["unassigned_reason"],
        "matching_workers_unavailable"
    );
}
