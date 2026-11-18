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

#[test]
fn opt_in_coverage_is_bound_to_the_compiled_package_and_snapshot() {
    use orchestrata_recipe_runtime::route_dispatch::{CoverageOutcome, MAX_COVERAGE_WORK};
    let mut members = files();
    asset(
        &mut members,
        "models/maintenance-cost.xml",
        MODEL.as_bytes().to_vec(),
    );
    let package = load_package(&pack(&members, false, true)).unwrap();
    let c = package.compile_route(None, DATA, 200).unwrap();
    let solved = package
        .compile_route(None, DATA, 200)
        .unwrap()
        .solve()
        .unwrap();
    let result = c
        .certify_coverage(&solved.solution, MAX_COVERAGE_WORK)
        .unwrap();
    assert_eq!(result.content_identity, solved.content_identity);
    assert_eq!(result.execution_identity, solved.execution_identity);
    assert_eq!(result.input_identity, solved.input_identity);
    assert_eq!(result.request_identity, solved.request_identity);
    let CoverageOutcome::Complete(cert) = result.coverage else {
        panic!("complete")
    };
    assert_eq!(
        (
            cert.incumbent_covered,
            cert.upper_bound,
            cert.maximum_coverage
        ),
        (3, 3, true)
    );
}

#[test]
fn stronger_package_certificate_retains_revision_and_input_identity() {
    use orchestrata_recipe_runtime::route_dispatch::{MAX_COVERAGE_WORK, TravelCoverageOutcome};
    let mut members = files();
    asset(
        &mut members,
        "models/maintenance-cost.xml",
        MODEL.as_bytes().to_vec(),
    );
    let package = load_package(&pack(&members, false, true)).unwrap();
    let c = package.compile_route(None, DATA, 200).unwrap();
    let solved = package
        .compile_route(None, DATA, 200)
        .unwrap()
        .solve()
        .unwrap();
    let checked = c
        .certify_coverage_with_travel(&solved.solution, MAX_COVERAGE_WORK)
        .unwrap();
    assert_eq!(checked.content_identity, solved.content_identity);
    assert_eq!(checked.execution_identity, solved.execution_identity);
    assert_eq!(checked.input_identity, solved.input_identity);
    assert_eq!(checked.request_identity, solved.request_identity);
    let TravelCoverageOutcome::Complete(cert) = checked.coverage else {
        panic!("complete")
    };
    assert_eq!((cert.incumbent_covered, cert.upper_bound), (3, 3));
}
