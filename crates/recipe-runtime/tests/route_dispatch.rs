use orchestrata_recipe_runtime::route_dispatch::{
    compile_route_model, parse_route_model, solve_route_model,
};
const MODEL: &str = r#"<route-dispatch version="1" id="dispatch" input-schema="directed-dispatch-v1" algorithm="input-order-insertion" time-unit="minutes" load-unit="kg" max-insertions="200"/>"#;
const DATA: &str = include_str!("fixtures/route-dispatch.json");
#[test]
fn bounded_native_route_interface_is_distinct_and_replays() {
    let spec = parse_route_model(MODEL).unwrap();
    let baseline = solve_route_model(compile_route_model(&spec, DATA, 200).unwrap()).unwrap();
    assert_eq!(
        (
            baseline.unassigned.len(),
            baseline.total_courier_minutes,
            baseline.evaluated_insertions
        ),
        (2, 29, 14)
    );
    let alternate =
        parse_route_model(&MODEL.replace("input-order-insertion", "two-order-insertion")).unwrap();
    let best = solve_route_model(compile_route_model(&alternate, DATA, 200).unwrap()).unwrap();
    assert_eq!(
        (
            best.unassigned.len(),
            best.total_courier_minutes,
            best.evaluated_insertions
        ),
        (1, 23, 37)
    );
    assert_eq!(best.guarantee, "verified-feasible-heuristic");
    for budget in [0, 1, 5, 14, 20, 36, 37, 200] {
        let a = solve_route_model(compile_route_model(&alternate, DATA, budget).unwrap()).unwrap();
        let b = solve_route_model(compile_route_model(&alternate, DATA, budget).unwrap()).unwrap();
        assert!(a.evaluated_insertions <= budget);
        assert_eq!(
            serde_json::to_value(a).unwrap(),
            serde_json::to_value(b).unwrap()
        );
    }
    assert!(compile_route_model(&spec, DATA, 201).is_err());
    assert!(orchestrata_recipe_runtime::mathematical::parse_model(MODEL).is_err());
    assert!(orchestrata_recipe_runtime::mathematical::parse_penalty_model(MODEL).is_err());
}
#[test]
fn rejects_unsupported_units_algorithms_schemas_and_unsafe_xml() {
    for bad in [
        MODEL.replace("minutes", "hours"),
        MODEL.replace("kg", "pounds"),
        MODEL.replace("input-order-insertion", "optimal"),
        MODEL.replace("version=\"1\"", "version=\"2\""),
        MODEL.replace("200", "20001"),
        MODEL.replace("directed-dispatch-v1", "any-json"),
        MODEL.replace("/>", "><program/></route-dispatch>"),
        format!("<!DOCTYPE x>{MODEL}"),
        format!("<?script execute?>{MODEL}"),
    ] {
        assert!(parse_route_model(&bad).is_err());
    }
    assert!(parse_route_model(&"x".repeat(32769)).is_err());
}
#[test]
fn strict_typed_records_and_no_guessed_travel() {
    let spec = parse_route_model(MODEL).unwrap();
    let mut data: serde_json::Value = serde_json::from_str(DATA).unwrap();
    data["travel_legs"] = serde_json::json!([]);
    let result =
        solve_route_model(compile_route_model(&spec, &data.to_string(), 200).unwrap()).unwrap();
    assert_eq!(result.unassigned.len(), 4);
    data["couriers"][0]["capacity_kg"] = serde_json::json!("5");
    assert!(compile_route_model(&spec, &data.to_string(), 200).is_err());
    assert!(compile_route_model(&spec, &" ".repeat(524289), 0).is_err());
    assert!(
        compile_route_model(
            &spec,
            r#"{"locations":[],"locations":[],"couriers":[],"shipments":[],"travel_legs":[]}"#,
            0
        )
        .is_err()
    );
}

#[test]
fn coverage_certificate_replays_fixture_and_preserves_output() {
    use orchestrata_recipe_runtime::route_dispatch::{
        CoverageOutcome, MAX_COVERAGE_WORK, certify_route_coverage,
    };
    for (model, expected_coverage) in [
        (MODEL.to_owned(), 2),
        (
            MODEL.replace("input-order-insertion", "two-order-insertion"),
            3,
        ),
    ] {
        let spec = parse_route_model(&model).unwrap();
        let compiled = compile_route_model(&spec, DATA, 200).unwrap();
        let mut result = solve_route_model(compile_route_model(&spec, DATA, 200).unwrap()).unwrap();
        let before = serde_json::to_vec(&result).unwrap();
        let CoverageOutcome::Complete(c) =
            certify_route_coverage(&compiled, &result, MAX_COVERAGE_WORK).unwrap()
        else {
            panic!("complete")
        };
        assert_eq!(
            (c.upper_bound, c.incumbent_covered, c.maximum_coverage),
            (3, expected_coverage, expected_coverage == 3)
        );
        assert_eq!(before, serde_json::to_vec(&result).unwrap());
        for budget in 0..=c.work {
            let attempt = certify_route_coverage(&compiled, &result, budget).unwrap();
            if budget < c.work {
                assert_eq!(attempt, CoverageOutcome::Incomplete { work: budget });
            } else {
                assert_eq!(attempt, CoverageOutcome::Complete(c.clone()));
            }
        }

        assert!(matches!(
            certify_route_coverage(&compiled, &result, 0).unwrap(),
            CoverageOutcome::Incomplete { work: 0 }
        ));
        result.routes[0].visits[0].service_end_minute += 1;
        assert!(certify_route_coverage(&compiled, &result, MAX_COVERAGE_WORK).is_err());
    }
}

#[test]
fn coverage_rejects_invalid_partition_and_replays_direct_arcs() {
    use orchestrata_recipe_runtime::route_dispatch::{MAX_COVERAGE_WORK, certify_route_coverage};
    let spec = parse_route_model(MODEL).unwrap();
    let c = compile_route_model(&spec, DATA, 200).unwrap();
    let mut result = solve_route_model(compile_route_model(&spec, DATA, 200).unwrap()).unwrap();
    result.unassigned.push(result.unassigned[0].clone());
    assert!(certify_route_coverage(&c, &result, MAX_COVERAGE_WORK).is_err());
    result.unassigned.pop();
    result.routes[0].visits[0].arrival_minute = 0;
    result.routes[0].visits[0].service_start_minute = 2;
    result.routes[0].visits[0].service_end_minute = 3;
    assert!(certify_route_coverage(&c, &result, MAX_COVERAGE_WORK).is_err());
}
