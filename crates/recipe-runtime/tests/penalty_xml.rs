use orchestrata_recipe_runtime::mathematical::{
    parse_model, parse_penalty_model, run_model, run_penalty_model,
};
use serde_json::Value;
const MODEL: &str = include_str!("fixtures/contact-center-v2.xml");
const RECORDS: &str = include_str!("fixtures/contact-center.json");

#[test]
fn versioned_penalty_result_has_explicit_units_and_no_coverage_score_alias() {
    let result = run_penalty_model(MODEL, RECORDS).unwrap();
    assert_eq!(
        (
            result.assigned,
            result.service_cost,
            result.unassigned_cost,
            result.total_cost
        ),
        (3, 600, 13100, 13700)
    );
    assert_eq!(result.schema_version, 2);
    assert_eq!(result.objective, "service-plus-unassigned-cost");
    assert_eq!(result.unit, "cents");
    assert_eq!(result.assignments[1].resource_id, None);
    assert_eq!(result.assignments[2].resource_id, None);
    assert_eq!(result.assignments[3].resource_id, None);
    assert!(
        serde_json::to_value(&result)
            .unwrap()
            .get("minimized_edge_sum")
            .is_none()
    );
    assert_eq!(result, run_penalty_model(MODEL, RECORDS).unwrap());
    assert!(parse_model(MODEL).is_err());
    assert!(run_model(MODEL, RECORDS).is_err());
    assert!(parse_penalty_model(&MODEL.replace("version=\"2\"", "version=\"1\"")).is_err());
}

#[test]
fn input_changes_penalty_and_availability_change_the_optimum() {
    let mut records: Value = serde_json::from_str(RECORDS).unwrap();
    records["requests"][1]["missed_service_cents"] = 9000.into();
    let result = run_penalty_model(MODEL, &records.to_string()).unwrap();
    assert_eq!(
        result.assignments[1].resource_id.as_deref(),
        Some("generalist")
    );
    assert_eq!(result.assignments[4].resource_id, None);
    for a in records["agents"].as_array_mut().unwrap() {
        a["available"] = false.into();
    }
    let result = run_penalty_model(MODEL, &records.to_string()).unwrap();
    assert_eq!(result.assigned, 0);
    assert_eq!(result.service_cost, 0);
    assert_eq!(result.total_cost, 40100);
}

#[test]
fn version_two_rejects_mixed_policies_missing_units_and_unsupplied_penalties() {
    for invalid in [
        MODEL.replace("order=\"single\"", "order=\"lexicographic\""),
        MODEL.replace(
            "</objectives>",
            "<maximize variable=\"routing\" operator=\"assigned-count\"/></objectives>",
        ),
        MODEL.replace("nullable=\"true\"", "nullable=\"false\""),
        MODEL.replace(" unit=\"cents\"", ""),
        MODEL.replace(
            "unit=\"cents\"",
            "service-unit=\"minutes\" penalty-unit=\"cents\"",
        ),
        MODEL.replace(
            "penalty-field=\"missed_service_cents\"",
            "penalty-field=\"unknown\"",
        ),
        MODEL.replace("service-plus-unassigned-cost", "edge-sum"),
        MODEL.replace("version=\"2\"", "version=\"3\""),
    ] {
        assert!(parse_penalty_model(&invalid).is_err(), "{invalid}");
    }
    let mut records: Value = serde_json::from_str(RECORDS).unwrap();
    records["requests"][0]
        .as_object_mut()
        .unwrap()
        .remove("missed_service_cents");
    assert!(run_penalty_model(MODEL, &records.to_string()).is_err());
    records["requests"][0]["missed_service_cents"] = Value::Null;
    assert!(run_penalty_model(MODEL, &records.to_string()).is_err());
    records["requests"][0]["missed_service_cents"] = (-1).into();
    assert!(run_penalty_model(MODEL, &records.to_string()).is_err());
}

#[test]
fn renamed_recipe_and_bindings_do_not_choose_scenario_behavior() {
    assert_eq!(
        run_penalty_model(MODEL, RECORDS).unwrap(),
        run_penalty_model(
            &MODEL.replace("contact-center-penalties", "unrelated-id"),
            RECORDS
        )
        .unwrap()
    );
    let model = MODEL
        .replace("missed_service_cents", "omit_fee")
        .replace("service_cents", "edge_fee");
    let records = RECORDS
        .replace("missed_service_cents", "omit_fee")
        .replace("service_cents", "edge_fee");
    assert_eq!(
        run_penalty_model(MODEL, RECORDS).unwrap(),
        run_penalty_model(&model, &records).unwrap()
    );
    assert!(run_penalty_model(&model, RECORDS).is_err());
}

#[test]
fn explicit_zero_penalties_are_allowed_but_unsafe_or_oversized_inputs_are_not() {
    let mut records: Value = serde_json::from_str(RECORDS).unwrap();
    for request in records["requests"].as_array_mut().unwrap() {
        request["missed_service_cents"] = 0.into();
    }
    let result = run_penalty_model(MODEL, &records.to_string()).unwrap();
    assert_eq!((result.assigned, result.total_cost), (0, 0));
    let bad = format!("<!DOCTYPE model [<!ENTITY x SYSTEM 'file:///tmp/no-read'>]>{MODEL}");
    assert!(parse_penalty_model(&bad).is_err());
    assert!(parse_penalty_model(&format!("<?unsupported instruction?>{MODEL}")).is_err());
    assert!(parse_penalty_model(&"x".repeat(32769)).is_err());
    assert!(run_penalty_model(MODEL, &" ".repeat(524289)).is_err());
    let row = records["requests"][0].clone();
    records["requests"] = vec![row; 129].into();
    assert!(run_penalty_model(MODEL, &records.to_string()).is_err());
}
