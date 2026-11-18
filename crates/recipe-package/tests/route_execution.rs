mod common;
use common::{asset, files, pack};
use orchestrata_recipe_package::load_package;
const MODEL: &str = r#"<route-dispatch version="1" id="dispatch" input-schema="directed-dispatch-v1" algorithm="two-order-insertion" time-unit="minutes" load-unit="kg" max-insertions="200"/>"#;
const DATA: &str = include_str!("../../recipe-runtime/tests/fixtures/route-dispatch.json");
#[test]
fn route_selection_is_explicit_and_identities_bind_snapshot_and_limit() {
    let original = load_package(common::PACKAGE).unwrap();
    assert!(original.compile_route(None, common::INPUT, 200).is_err());
    let mut members = files();
    asset(
        &mut members,
        "models/maintenance-cost.xml",
        MODEL.as_bytes().to_vec(),
    );
    let package = load_package(&pack(&members, false, true)).unwrap();
    assert!(package.compile(None, DATA).is_err());
    assert!(package.compile_penalty(None, DATA).is_err());
    let result = package
        .compile_route(None, DATA, 200)
        .unwrap()
        .solve()
        .unwrap();
    assert_eq!(result.solution.unassigned.len(), 1);
    let lower = package
        .compile_route(None, DATA, 37)
        .unwrap()
        .solve()
        .unwrap();
    assert_eq!(result.input_identity, lower.input_identity);
    assert_eq!(result.execution_identity, lower.execution_identity);
    assert_ne!(result.request_identity, lower.request_identity);
    let changed = package
        .compile_route(None, &format!("{DATA}\n"), 200)
        .unwrap()
        .solve()
        .unwrap();
    assert_ne!(result.input_identity, changed.input_identity);
    assert_eq!(result.execution_identity, changed.execution_identity);
    assert!(package.compile(Some("delay"), common::INPUT).is_ok());
    assert!(package.compile_route(Some("delay"), DATA, 200).is_err());
    asset(
        &mut members,
        "models/maintenance-cost.xml",
        MODEL.replace("minutes", "hours").into_bytes(),
    );
    assert!(load_package(&pack(&members, false, true)).is_err());
}
