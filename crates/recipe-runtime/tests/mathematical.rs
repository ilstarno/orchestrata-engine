use orchestrata_recipe_runtime::mathematical::{ModelError, run_model};
use serde_json::{Value, json};
use std::{path::Path, process::Command};
const COST: &str = include_str!("fixtures/math/cost.xml");
const DELAY: &str = include_str!("fixtures/math/delay.xml");
const RECORDS: &str = include_str!("fixtures/math/records.json");
fn records() -> Value {
    serde_json::from_str(RECORDS).unwrap()
}

#[test]
fn same_binary_same_records_different_models_produce_exact_different_optima() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let invoke = |model: &str| {
        let p = Command::new(env!("CARGO_BIN_EXE_solve-model"))
            .arg(root.join(format!("tests/fixtures/math/{model}.xml")))
            .arg(root.join("tests/fixtures/math/records.json"))
            .output()
            .unwrap();
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        serde_json::from_slice::<Value>(&p.stdout).unwrap()
    };
    let cost = invoke("cost");
    let delay = invoke("delay");
    assert_eq!(
        cost,
        json!({"assigned":2,"minimized_edge_sum":1,"assignments":[
        {"item_id":"repair","resource_id":"a"},{"item_id":"inspect","resource_id":"b"}]})
    );
    assert_eq!(
        delay,
        json!({"assigned":2,"minimized_edge_sum":1,"assignments":[
        {"item_id":"repair","resource_id":"b"},{"item_id":"inspect","resource_id":"a"}]})
    );
    assert_ne!(cost, delay);
}

#[test]
fn ids_bindings_inputs_parameters_and_constraints_have_explicit_semantics() {
    let baseline = run_model(COST, RECORDS).unwrap();
    assert_eq!(
        baseline,
        run_model(
            &COST.replace("id=\"maintenance\"", "id=\"unrelated-id\""),
            RECORDS
        )
        .unwrap()
    );
    let renamed = COST.replace("field=\"orders\"", "field=\"tickets\"");
    assert!(run_model(&renamed, RECORDS).is_err());
    assert_eq!(
        baseline,
        run_model(&renamed, &RECORDS.replace("\"orders\"", "\"tickets\"")).unwrap()
    );
    let renamed = COST
        .replace("slots", "quota")
        .replace("tasks", "arbitrary-table")
        .replace("assignment", "selection");
    assert_eq!(
        baseline,
        run_model(&renamed, &RECORDS.replace("slots", "quota")).unwrap()
    );
    let mut data = records();
    data["crew"][0]["active"] = json!(false);
    assert_eq!(run_model(COST, &data.to_string()).unwrap().assigned, 1);
    let larger = COST.replace("value=\"1\"", "value=\"2\"");
    let result = run_model(&larger, &data.to_string()).unwrap();
    assert_eq!((result.assigned, result.minimized_edge_sum), (2, Some(8)));
    assert!(
        result
            .assignments
            .iter()
            .all(|a| a.resource_id.as_deref() == Some("b"))
    );
    data["crew"][1]["slots"] = json!(0);
    assert_eq!(run_model(&larger, &data.to_string()).unwrap().assigned, 0);
    data = records();
    data["options"].as_array_mut().unwrap().remove(0);
    let result = run_model(COST, &data.to_string()).unwrap();
    assert_eq!((result.assigned, result.minimized_edge_sum), (2, Some(13)));
    let conflict = COST.replace(
        "</constraints>",
        "<availability variable=\"assignment\" field=\"active\" value=\"false\"/></constraints>",
    );
    assert_eq!(run_model(&conflict, RECORDS).unwrap().assigned, 0);
    assert_eq!(
        run_model(
            &conflict.replace("nullable=\"true\"", "nullable=\"false\""),
            RECORDS
        ),
        Err(ModelError::Infeasible)
    );
}

#[test]
fn unsupported_objective_order_or_operators_are_rejected_not_skipped() {
    let maximum = "<maximize variable=\"assignment\" operator=\"assigned-count\"/>";
    let minimum = "<minimize variable=\"assignment\" operator=\"edge-sum\" field=\"price\"/>";
    for xml in [
        COST.replace(
            &(maximum.to_owned() + minimum),
            &(minimum.to_owned() + maximum),
        ),
        COST.replace(maximum, ""),
        COST.replace("edge-sum", "travel-time"),
        COST.replace("lexicographic", "weighted"),
        COST.replace("kind=\"reference\"", "kind=\"program\""),
        COST.replace("field=\"price\"/>", "field=\"person\"/>"),
        COST.replace("variable=\"assignment\"", "variable=\"missing\""),
        COST.replace("limit=\"ceiling\"", "limit=\"unknown\""),
        COST.replace("type=\"u32\"", "type=\"float\""),
        COST.replace("version=\"1\"", "version=\"2\""),
        COST.replace("<constraints>", "<constraints><program/>"),
    ] {
        assert!(
            run_model(&xml, RECORDS).is_err(),
            "unsupported model accepted"
        );
    }
    let coverage_only = COST.replace(minimum, "");
    let result = run_model(&coverage_only, RECORDS).unwrap();
    assert_eq!(result.assigned, 2);
    assert_eq!(result.minimized_edge_sum, None);
}

#[test]
fn strict_shape_and_reference_validation_rejects_ambiguous_or_invalid_inputs() {
    for table in ["orders", "crew", "options"] {
        let mut data = records();
        data[table][0] = json!(["repair"]);
        assert!(run_model(COST, &data.to_string()).is_err());
    }
    for (table, key, value) in [
        ("crew", "active", json!(1)),
        ("crew", "slots", json!(-1)),
        ("options", "price", json!(1_000_001)),
        ("options", "person", json!("missing")),
        ("orders", "extra", json!(true)),
    ] {
        let mut d = records();
        d[table][0][key] = value;
        assert!(run_model(COST, &d.to_string()).is_err());
    }
    let mut d = records();
    d["orders"][1]["key"] = json!("repair");
    assert!(run_model(COST, &d.to_string()).is_err());
    let mut d = records();
    let row = d["options"][0].clone();
    d["options"].as_array_mut().unwrap().push(row);
    assert!(run_model(COST, &d.to_string()).is_err());
    assert!(
        run_model(
            COST,
            &RECORDS.replace(
                "\"key\": \"repair\"",
                "\"key\": \"repair\", \"key\": \"repair\""
            )
        )
        .is_err()
    );
    assert!(run_model(COST, &RECORDS.replacen('{', "{\"orders\":[],", 1)).is_err());
}

#[test]
fn unsafe_and_oversized_inputs_fail_closed_under_fixed_bounds() {
    for xml in [
        format!("<!DOCTYPE model [<!ENTITY e SYSTEM 'https://example.invalid/x'>]>{COST}"),
        format!("<?run unsafe?>{COST}"),
        COST.replace("<model ", "<model xmlns=\"urn:foreign\" "),
        COST.replace(
            "</parameters>",
            &format!("{}</parameters>", "<x/>".repeat(300)),
        ),
        format!("{}{}", COST, " ".repeat(32769)),
    ] {
        assert!(run_model(&xml, RECORDS).is_err());
    }
    assert_eq!(
        run_model(COST, &" ".repeat(524289)),
        Err(ModelError::ResourceLimit)
    );
    let mut d = records();
    d["orders"] = json!(
        (0..129)
            .map(|i| json!({"key":format!("i-{i}")}))
            .collect::<Vec<_>>()
    );
    assert!(run_model(COST, &d.to_string()).is_err());
    d["orders"] = json!(
        (0..33)
            .map(|i| json!({"key":format!("i-{i}")}))
            .collect::<Vec<_>>()
    );
    assert_eq!(
        run_model(COST, &d.to_string()),
        Err(ModelError::ResourceLimit)
    );
    let mut d = records();
    d["crew"][0]["slots"] = json!({"program":"exit"});
    assert!(run_model(COST, &d.to_string()).is_err());
}

// Enumerate every optional assignment, independently of both matching implementations.
fn oracle(data: &Value, field: &str, ceiling: u64) -> (usize, u64) {
    let mut best = (0, u64::MAX);
    for encoding in 0..9 {
        let choices = [encoding % 3, encoding / 3];
        let mut used = [0; 2];
        let mut count = 0;
        let mut sum = 0;
        let mut valid = true;
        for (i, &w) in choices.iter().enumerate() {
            if w == 2 {
                continue;
            }
            let id = &data["crew"][w]["key"];
            let edge = data["options"]
                .as_array()
                .unwrap()
                .iter()
                .find(|e| e["task"] == data["orders"][i]["key"] && e["person"] == *id);
            if let Some(edge) = edge {
                sum += edge[field].as_u64().unwrap();
            } else {
                valid = false;
            }
            valid &= data["crew"][w]["active"] == true;
            used[w] += 1;
            count += 1;
        }
        for (w, &n) in used.iter().enumerate() {
            valid &= n <= data["crew"][w]["slots"].as_u64().unwrap().min(ceiling);
        }
        if valid && (count > best.0 || (count == best.0 && sum < best.1)) {
            best = (count, sum);
        }
    }
    best
}
#[test]
fn compiled_objectives_match_independent_exact_oracle_across_recalculations() {
    let mut cases = 0;
    for mask in 0..16 {
        for available in 0..4 {
            for cap in 0..3 {
                for ceiling in 1..3 {
                    let mut d = records();
                    d["crew"][0]["active"] = json!(available & 1 != 0);
                    d["crew"][1]["active"] = json!(available & 2 != 0);
                    d["crew"][0]["slots"] = json!(cap);
                    d["crew"][1]["slots"] = json!((cap + 1) % 3);
                    for (i, e) in d["options"].as_array_mut().unwrap().iter_mut().enumerate() {
                        e["price"] = json!((mask + i * 3) % 7);
                        e["delay"] = json!((mask * 3 + i) % 5);
                    }
                    let edges = d["options"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| mask & (1 << i) != 0)
                        .map(|(_, e)| e.clone())
                        .collect::<Vec<_>>();
                    d["options"] = json!(edges);
                    for (xml, field) in [(COST, "price"), (DELAY, "delay")] {
                        let xml = xml.replace("value=\"1\"", &format!("value=\"{ceiling}\""));
                        let result = run_model(&xml, &d.to_string()).unwrap();
                        let exact = oracle(&d, field, ceiling);
                        assert_eq!((result.assigned, result.minimized_edge_sum.unwrap()), exact);
                        let mut used = [0u64; 2];
                        let mut sum = 0;
                        let mut count = 0;
                        for (i, a) in result.assignments.iter().enumerate() {
                            assert_eq!(Some(a.item_id.as_str()), d["orders"][i]["key"].as_str());
                            if let Some(id) = &a.resource_id {
                                let w = d["crew"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .position(|r| r["key"].as_str() == Some(id))
                                    .unwrap();
                                assert_eq!(d["crew"][w]["active"], true);
                                used[w] += 1;
                                count += 1;
                                let edge = d["options"]
                                    .as_array()
                                    .unwrap()
                                    .iter()
                                    .find(|e| {
                                        e["task"].as_str() == Some(&a.item_id)
                                            && e["person"].as_str() == Some(id)
                                    })
                                    .unwrap();
                                sum += edge[field].as_u64().unwrap();
                            }
                        }
                        assert_eq!((count, sum), exact);
                        for (w, &n) in used.iter().enumerate() {
                            assert!(n <= d["crew"][w]["slots"].as_u64().unwrap().min(ceiling));
                        }
                        cases += 1;
                    }
                }
            }
        }
    }
    assert_eq!(cases, 768);
}
