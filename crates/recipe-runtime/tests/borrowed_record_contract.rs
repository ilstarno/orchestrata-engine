use orchestrata_recipe_runtime::mathematical::{ModelError, run_model};

const COST: &str = include_str!("fixtures/math/cost.xml");
const DELAY: &str = include_str!("fixtures/math/delay.xml");
const DATA: &str = include_str!("fixtures/math/records.json");

#[test]
fn escaped_keys_and_values_survive_raw_slice_deserialization() {
    let escaped = DATA
        .replace("\"orders\"", r#""\u006frders""#)
        .replace("\"key\"", r#""k\u0065y""#)
        .replace("\"repair\"", r#""r\u0065pair""#);
    for model in [COST, DELAY] {
        assert_eq!(run_model(model, &escaped), run_model(model, DATA));
    }
    let equivalent_duplicate = escaped.replacen(
        r#""k\u0065y": "r\u0065pair""#,
        r#""k\u0065y": "r\u0065pair", "key": "repair""#,
        1,
    );
    assert_ne!(equivalent_duplicate, escaped);
    assert_eq!(
        run_model(COST, &equivalent_duplicate),
        Err(ModelError::InvalidInput)
    );
}

#[test]
fn large_integer_costs_zero_capacity_and_inactive_resources_keep_exact_ordered_output() {
    let data = r#"{
        "orders":[{"key":"a"},{"key":"b"},{"key":"unavailable"}],
        "crew":[{"key":"active","slots":1,"active":true},
                {"key":"zero","slots":0,"active":true},
                {"key":"inactive","slots":2,"active":false}],
        "options":[{"task":"a","person":"active","price":1000000,"delay":7},
                   {"task":"b","person":"active","price":999999,"delay":8},
                   {"task":"b","person":"zero","price":0,"delay":0},
                   {"task":"unavailable","person":"inactive","price":0,"delay":0}]} "#;
    for (model, cost, chosen) in [(COST, 999_999, 1), (DELAY, 7, 0)] {
        let solved = run_model(model, data).unwrap();
        assert_eq!(
            (solved.assigned, solved.minimized_edge_sum),
            (1, Some(cost))
        );
        assert_eq!(
            solved
                .assignments
                .iter()
                .map(|a| a.item_id.as_str())
                .collect::<Vec<_>>(),
            ["a", "b", "unavailable"]
        );
        for (i, row) in solved.assignments.iter().enumerate() {
            assert_eq!(
                row.resource_id.as_deref(),
                (i == chosen).then_some("active")
            );
        }
    }
}
