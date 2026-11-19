mod common;
use common::*;
use orchestrata_recipe_package::{ControlledModelError, load_package};

#[test]
fn controlled_package_preserves_identity_and_results_or_returns_cancellation() {
    let package = load_package(PACKAGE).unwrap();
    for mode in ["cost", "delay"] {
        let baseline = package.compile(Some(mode), INPUT).unwrap().solve().unwrap();
        let mut checks = 0;
        let actual = package
            .compile(Some(mode), INPUT)
            .unwrap()
            .solve_controlled(|| {
                checks += 1;
                Ok::<(), usize>(())
            })
            .unwrap();
        assert_eq!(actual.solution, baseline.solution);
        assert_eq!(actual.content_identity, baseline.content_identity);
        assert_eq!(actual.execution_identity, baseline.execution_identity);
        for stop in [1, checks / 2, checks] {
            let mut calls = 0;
            let result = package
                .compile(Some(mode), INPUT)
                .unwrap()
                .solve_controlled(|| {
                    calls += 1;
                    if calls == stop { Err(stop) } else { Ok(()) }
                });
            assert!(matches!(result, Err(ControlledModelError::Cancelled(n)) if n == stop));
            assert_eq!(calls, stop);
        }
    }
}

#[test]
fn coverage_only_is_explicitly_unsupported_without_affecting_original_api() {
    let mut entries = files();
    let model_path = "models/maintenance-cost.xml";
    let xml = String::from_utf8(entries[model_path].clone()).unwrap();
    let start = xml.find("<minimize").unwrap();
    let end = start + xml[start..].find("/>").unwrap() + 2;
    let mut changed = xml;
    changed.replace_range(start..end, "");
    asset(&mut entries, model_path, changed.into_bytes());
    let bytes = pack(&entries, false, true);
    let package = load_package(&bytes).unwrap();
    assert!(
        package
            .compile(Some("cost"), INPUT)
            .unwrap()
            .solve()
            .is_ok()
    );
    assert!(matches!(
        package
            .compile(Some("cost"), INPUT)
            .unwrap()
            .solve_controlled(|| Ok::<(), ()>(())),
        Err(ControlledModelError::UnsupportedGoal)
    ));
}
