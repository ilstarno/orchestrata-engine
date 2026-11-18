mod common;
use common::{INPUT, asset, files, pack};
use orchestrata_recipe_package::load_package;
const MODEL: &str = include_str!("../../recipe-runtime/tests/fixtures/contact-center-v2.xml");
const RECORDS: &str = include_str!("../../recipe-runtime/tests/fixtures/contact-center.json");

#[test]
fn explicit_package_v2_method_preserves_v1_default_and_result_contracts() {
    let original = load_package(common::PACKAGE).unwrap();
    assert!(original.compile_penalty(None, INPUT).is_err());
    let old = original.compile(None, INPUT).unwrap().solve().unwrap();
    let mut members = files();
    asset(
        &mut members,
        "models/maintenance-cost.xml",
        MODEL.as_bytes().to_vec(),
    );
    let package = load_package(&pack(&members, false, true)).unwrap();
    // Default remains exactly the manifest-selected entrypoint; no fallback.
    assert!(package.compile(None, RECORDS).is_err());
    let result = package
        .compile_penalty(None, RECORDS)
        .unwrap()
        .solve()
        .unwrap();
    assert_eq!(result.solution.total_cost, 13700);
    assert_ne!(result.execution_identity, old.execution_identity);
    assert!(package.compile_penalty(Some("delay"), RECORDS).is_err());
    assert!(package.compile(Some("delay"), INPUT).is_ok());
    let reordered = load_package(&pack(&members, true, false)).unwrap();
    let replay = reordered
        .compile_penalty(None, RECORDS)
        .unwrap()
        .solve()
        .unwrap();
    assert_eq!(result.content_identity, replay.content_identity);
    assert_eq!(result.execution_identity, replay.execution_identity);
    assert_eq!(result.solution, replay.solution);
}

#[test]
fn package_validates_nondefault_v2_models_and_declared_units_before_exposing_any() {
    let mut members = files();
    asset(
        &mut members,
        "models/maintenance-delay.xml",
        MODEL.replace("unit=\"cents\"", "unit=\"\"").into_bytes(),
    );
    assert!(load_package(&pack(&members, false, true)).is_err());
    asset(
        &mut members,
        "models/maintenance-delay.xml",
        MODEL.as_bytes().to_vec(),
    );
    assert!(load_package(&pack(&members, false, true)).is_ok());
}
