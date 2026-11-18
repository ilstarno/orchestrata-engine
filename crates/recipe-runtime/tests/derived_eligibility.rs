use orchestrata_recipe_runtime::mathematical::{
    ModelError, parse_derived_model, run_derived_model, run_model, run_penalty_model,
};
use serde_json::{Value, json};
const XML: &str = include_str!("fixtures/derived/door-allocation.xml");
const RECORDS: &str = include_str!("fixtures/derived/records.json");
fn two_rules() -> String {
    XML.replace(
        "  </decision>",
        "    <equal item-field=\"interface\" resource-field=\"interface\"/>\n  </decision>",
    )
}
fn exact(data: &Value, second: bool) -> (usize, usize) {
    let items = data["trailers"].as_array().unwrap();
    let resources = data["doors"].as_array().unwrap();
    let edges = items
        .iter()
        .map(|item| {
            resources
                .iter()
                .map(|r| {
                    item["temperature"] == r["temperature"]
                        && (!second || item["interface"] == r["interface"])
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let count = edges.iter().flatten().filter(|&&e| e).count();
    let mut best = 0;
    for mut candidate in 0..(resources.len() + 1).pow(items.len() as u32) {
        let mut loads = vec![0; resources.len()];
        let mut assigned = 0;
        let mut valid = true;
        for row in &edges {
            let choice = candidate % (resources.len() + 1);
            candidate /= resources.len() + 1;
            if choice == resources.len() {
                continue;
            }
            loads[choice] += 1;
            assigned += 1;
            valid &= row[choice]
                && resources[choice]["available"] == true
                && loads[choice] <= resources[choice]["capacity"].as_u64().unwrap();
        }
        if valid {
            best = best.max(assigned);
        }
    }
    (count, best)
}
fn check(data: &Value, second: bool) -> usize {
    let xml = if second { two_rules() } else { XML.to_owned() };
    let result = run_derived_model(&xml, &data.to_string()).unwrap();
    assert_eq!(result.assigned, exact(data, second).1);
    assert_eq!(result.minimized_edge_sum, None);
    let mut loads = [0; 3];
    for (i, a) in result.assignments.iter().enumerate() {
        assert_eq!(a.item_id, data["trailers"][i]["id"].as_str().unwrap());
        if let Some(id) = &a.resource_id {
            let r = data["doors"]
                .as_array()
                .unwrap()
                .iter()
                .position(|r| r["id"] == *id)
                .unwrap();
            loads[r] += 1;
            assert_eq!(
                data["trailers"][i]["temperature"],
                data["doors"][r]["temperature"]
            );
            if second {
                assert_eq!(
                    data["trailers"][i]["interface"],
                    data["doors"][r]["interface"]
                );
            }
            assert_eq!(data["doors"][r]["available"], true);
            assert!(loads[r] <= data["doors"][r]["capacity"].as_u64().unwrap());
        }
    }
    assert_eq!(result.assigned as u64, loads.iter().sum::<u64>());
    result.assigned
}
#[test]
fn declared_predicates_change_real_coverage_and_aliases_do_not() {
    let data: Value = serde_json::from_str(RECORDS).unwrap();
    assert_eq!(exact(&data, false), (6, 4));
    assert_eq!(exact(&data, true), (5, 3));
    assert_eq!(check(&data, false), 4);
    assert_eq!(check(&data, true), 3);
    let mut xml = two_rules();
    let mut records = RECORDS.to_owned();
    for (a, b) in [
        ("single-window-allocation", "renamed"),
        ("trailers", "requests"),
        ("doors", "agents"),
        ("loads", "tasks"),
        ("slots", "pool"),
        ("allocation", "chosen"),
        ("temperature", "region"),
        ("interface", "language"),
        ("capacity", "quota"),
        ("available", "online"),
    ] {
        xml = xml.replace(&format!("\"{a}\""), &format!("\"{b}\""));
        records = records.replace(&format!("\"{a}\""), &format!("\"{b}\""));
    }
    assert_eq!(
        run_derived_model(&xml, &records).unwrap(),
        run_derived_model(&two_rules(), RECORDS).unwrap()
    );
}
#[test]
fn exact_enumeration_over_changed_capacities_availability_and_inputs() {
    for cap in 0..4 {
        for mask in 0..8 {
            for second in [false, true] {
                let mut data: Value = serde_json::from_str(RECORDS).unwrap();
                data["doors"][2]["capacity"] = json!(cap);
                for r in 0..3 {
                    data["doors"][r]["available"] = json!(mask & (1 << r) != 0);
                }
                check(&data, second);
                data["trailers"][2]["interface"] = json!("B");
                check(&data, second);
            }
        }
    }
}
#[test]
fn explicit_budget_is_preflighted_without_predicate_short_circuit() {
    assert!(matches!(
        run_derived_model(
            &XML.replace("pair-limit=\"12\"", "pair-limit=\"11\""),
            RECORDS
        ),
        Err(ModelError::ResourceLimit)
    ));
    assert!(matches!(
        run_derived_model(
            &two_rules().replace("comparison-limit=\"24\"", "comparison-limit=\"23\""),
            RECORDS
        ),
        Err(ModelError::ResourceLimit)
    ));
    for _ in 0..3 {
        assert_eq!(
            run_derived_model(&two_rules(), RECORDS).unwrap().assigned,
            3
        );
    }
}
#[test]
fn malformed_shapes_types_unknown_fields_and_mixed_domains_fail_closed() {
    for xml in [
        XML.replace("item-field=\"temperature\"", "item-field=\"missing\""),
        XML.replace(
            "resource-field=\"temperature\"",
            "resource-field=\"capacity\"",
        ),
        XML.replace("<equal ", "<less-than "),
        XML.replace("domain=\"derived\"", "domain=\"edges\""),
        XML.replace("pair-limit=\"12\"", "pair-limit=\"1025\""),
        XML.replace("comparison-limit=\"24\"", "comparison-limit=\"8193\""),
        XML.replace(" pair-limit=\"12\"", ""),
        XML.replace("domain=\"derived\"", "domain=\"derived\" edge-item=\"id\""),
        XML.replace(
            "<equal item-field=\"temperature\" resource-field=\"temperature\"/>",
            "",
        ),
        XML.replace(
            "<maximize variable=\"allocation\" operator=\"assigned-count\"/>",
            "<minimize variable=\"allocation\" operator=\"edge-sum\" field=\"cost\"/>",
        ),
    ] {
        assert!(parse_derived_model(&xml).is_err(), "{xml}");
    }
    for modify in 0..5 {
        let mut data: Value = serde_json::from_str(RECORDS).unwrap();
        match modify {
            0 => data["trailers"][0]["temperature"] = json!(true),
            1 => data["doors"][0]["capacity"] = json!("1"),
            2 => data["doors"][0]["extra"] = json!(0),
            3 => data["edges"] = json!([]),
            _ => data["trailers"][0] = json!([]),
        }
        assert!(run_derived_model(XML, &data.to_string()).is_err());
    }
    assert!(run_model(XML, RECORDS).is_err());
    assert!(run_penalty_model(XML, RECORDS).is_err());
    assert!(parse_derived_model(&XML.replace("version=\"3\"", "version=\"1\"")).is_err());
    assert!(parse_derived_model(&format!("<!DOCTYPE model>{XML}")).is_err());
    assert!(parse_derived_model(&format!("<?unsafe test?>{XML}")).is_err());
    assert!(parse_derived_model(&"x".repeat(32769)).is_err());
    assert!(run_derived_model(XML, &" ".repeat(524289)).is_err());
}
#[test]
fn unsigned_boolean_comparisons_and_empty_inputs_are_typed() {
    for ty in ["u32", "bool"] {
        let xml = XML.replace(
            "name=\"temperature\" type=\"id\"",
            &format!("name=\"temperature\" type=\"{ty}\""),
        );
        let mut data: Value = serde_json::from_str(RECORDS).unwrap();
        for table in ["trailers", "doors"] {
            for row in data[table].as_array_mut().unwrap() {
                let cold = row["temperature"] == "cold";
                row["temperature"] = if ty == "bool" {
                    json!(cold)
                } else {
                    json!(if cold { 1_000_000 } else { 0 })
                };
            }
        }
        assert_eq!(
            run_derived_model(&xml, &data.to_string()).unwrap().assigned,
            4
        );
        data["trailers"][0]["temperature"] = json!("wrong");
        assert!(run_derived_model(&xml, &data.to_string()).is_err());
    }
    let mut data: Value = serde_json::from_str(RECORDS).unwrap();
    data["trailers"] = json!([]);
    assert_eq!(
        run_derived_model(XML, &data.to_string()).unwrap().assigned,
        0
    );
    data["doors"] = json!([]);
    assert_eq!(
        run_derived_model(
            &XML.replace("pair-limit=\"12\"", "pair-limit=\"0\"")
                .replace("comparison-limit=\"24\"", "comparison-limit=\"0\""),
            &data.to_string()
        )
        .unwrap()
        .assigned,
        0
    );
}
#[test]
fn hard_row_pair_predicate_bounds_and_non_nullable_infeasibility() {
    let mut data: Value = serde_json::from_str(RECORDS).unwrap();
    data["trailers"] = (0..32)
        .map(|i| json!({"id":format!("t{i}"),"temperature":"cold","interface":"A"}))
        .collect();
    data["doors"]=(0..32).map(|i|json!({"id":format!("d{i}"),"temperature":"cold","interface":"A","capacity":1,"available":true})).collect();
    let rule = "<equal item-field=\"temperature\" resource-field=\"temperature\"/>";
    let xml = XML
        .replace("pair-limit=\"12\"", "pair-limit=\"1024\"")
        .replace("comparison-limit=\"24\"", "comparison-limit=\"8192\"")
        .replace(rule, &rule.repeat(8));
    assert_eq!(
        run_derived_model(&xml, &data.to_string()).unwrap().assigned,
        32
    );
    assert!(parse_derived_model(&xml.replace(&rule.repeat(8), &rule.repeat(9))).is_err());
    data["trailers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"id":"extra","temperature":"cold","interface":"A"}));
    assert!(matches!(
        run_derived_model(&xml, &data.to_string()),
        Err(ModelError::ResourceLimit)
    ));
    assert!(matches!(
        run_derived_model(
            &two_rules().replace("nullable=\"true\"", "nullable=\"false\""),
            RECORDS
        ),
        Err(ModelError::Infeasible)
    ));
}
