use orchestrata_recipe_runtime::mathematical::{ModelError, run_model};
const PARCEL: &str = include_str!("fixtures/typed/parcel-cost.xml");
const REVIEW: &str = include_str!("fixtures/typed/review-time.xml");
const PARCELS: &str = include_str!("fixtures/typed/parcel-records.json");
const REVIEWS: &str = include_str!("fixtures/typed/review-records.json");
#[test]
fn authored_vocabulary_and_goals_keep_existing_exact_semantics() {
    for (xml, data, weights, winner) in [
        (PARCEL, PARCELS, [[0, 7], [6, 1]], [0, 1]),
        (REVIEW, REVIEWS, [[9, 0], [1, 8]], [1, 0]),
    ] {
        let mut exact = (0, 0);
        let mut exact_plan = [None, None];
        for a in [None, Some(0), Some(1)] {
            for b in [None, Some(0), Some(1)] {
                if a.is_some() && a == b {
                    continue;
                }
                let covered = usize::from(a.is_some()) + usize::from(b.is_some());
                let cost = a.map_or(0, |r| weights[0][r]) + b.map_or(0, |r| weights[1][r]);
                if covered > exact.0 || (covered == exact.0 && cost < exact.1) {
                    exact = (covered, cost);
                    exact_plan = [a, b];
                }
            }
        }
        let result = run_model(xml, data).unwrap();
        assert_eq!(
            (result.assigned, result.minimized_edge_sum),
            (exact.0, Some(exact.1))
        );
        assert_eq!(exact_plan, winner.map(Some));
        assert_eq!(
            result
                .assignments
                .iter()
                .map(|a| a.resource_id.as_deref())
                .collect::<Vec<_>>(),
            winner.map(|i| Some(["x", "y"][i]))
        );
        assert_eq!(
            run_model(&xml.replace("edge-sum", "arbitrary-expression"), data),
            Err(ModelError::UnsupportedSemantics)
        );
    }
    assert!(run_model(PARCEL, REVIEWS).is_err());
    assert!(run_model(REVIEW, PARCELS).is_err());
    assert!(
        run_model(
            &PARCEL.replace("field=\"fee\"/>", "field=\"destination\"/>"),
            PARCELS
        )
        .is_err()
    );
    let mut parcels: serde_json::Value = serde_json::from_str(PARCELS).unwrap();
    parcels["lockers"][0]["slots"] = serde_json::json!("2");
    assert!(run_model(PARCEL, &parcels.to_string()).is_err());
    let mut reviews: serde_json::Value = serde_json::from_str(REVIEWS).unwrap();
    reviews["editors"][0]["blocked"] = serde_json::json!(0);
    assert!(run_model(REVIEW, &reviews.to_string()).is_err());
    assert!(
        run_model(
            REVIEW,
            &REVIEWS.replace("\"turnaround\"", "\"wrong-column\"")
        )
        .is_err()
    );
}
