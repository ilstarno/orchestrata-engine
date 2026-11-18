mod common;
use common::*;
use orchestrata_recipe_package::{DistributionTier, PackageError, load_package};
use serde_json::{Value, json};

#[test]
fn real_multi_model_archive_runs_reference_optima_and_default() {
    let p = load_package(PACKAGE).unwrap();
    assert_eq!(p.assets().count(), 3);
    assert_eq!(p.entrypoints().count(), 2);
    assert_eq!(p.default_entrypoint(), "cost");
    assert_eq!(
        p.metadata().translations["sq"].title,
        "Shpërndarja ditore e punëve të mirëmbajtjes"
    );
    let cost = p.compile(None, INPUT).unwrap().solve().unwrap();
    let delay = p.compile(Some("delay"), INPUT).unwrap().solve().unwrap();
    assert_eq!(
        serde_json::to_value(&cost.solution).unwrap(),
        serde_json::from_str::<Value>(include_str!("fixtures/expected/cost.json")).unwrap()
    );
    assert_eq!(
        serde_json::to_value(&delay.solution).unwrap(),
        serde_json::from_str::<Value>(include_str!("fixtures/expected/delay.json")).unwrap()
    );
    assert_ne!(cost.solution, delay.solution);
    assert_ne!(cost.execution_identity, delay.execution_identity);
    assert!(matches!(
        p.compile(Some("models/maintenance-cost.xml"), INPUT),
        Err(PackageError::Entrypoint)
    ));
    assert!(matches!(
        p.compile(Some("legal/terms.xml"), INPUT),
        Err(PackageError::Entrypoint)
    ));
}

#[test]
fn canonical_identities_match_independent_python_vectors_and_ignore_zip_layout() {
    let p = load_package(PACKAGE).unwrap();
    let expected: Value = serde_json::from_str(include_str!("fixtures/identities.json")).unwrap();
    assert_eq!(p.content_identity().to_hex(), expected["content_identity"]);
    for mode in ["cost", "delay"] {
        assert_eq!(
            p.compile(Some(mode), INPUT)
                .unwrap()
                .execution_identity()
                .to_hex(),
            expected[mode]
        );
    }
    for reverse in [true, false] {
        for stored in [true, false] {
            let copy = load_package(&pack(&files(), reverse, stored)).unwrap();
            assert_eq!(p.content_identity(), copy.content_identity());
            assert_eq!(
                p.compile(None, INPUT).unwrap().execution_identity(),
                copy.compile(None, INPUT).unwrap().execution_identity()
            );
        }
    }
}

#[test]
fn display_metadata_and_distribution_are_not_execution_identity_or_authorization() {
    let original = load_package(PACKAGE).unwrap();
    let mut f = files();
    manifest(&mut f, |s| {
        s.replace("Daily maintenance allocation", "Different display title")
            .replace(
                "Synthetic test fixture (publisher unspecified)",
                "Different claimed author",
            )
            .replace("tier=\"free\"", "tier=\"paid\"")
            .replace(
                "expression=\"Apache-2.0\"",
                "expression=\"MIT OR Apache-2.0\"",
            )
    });
    let changed = load_package(&pack(&f, false, false)).unwrap();
    assert_eq!(changed.metadata().distribution, DistributionTier::Paid);
    assert_ne!(original.content_identity(), changed.content_identity());
    let a = original.compile(None, INPUT).unwrap().solve().unwrap();
    let b = changed.compile(None, INPUT).unwrap().solve().unwrap();
    assert_eq!(a.execution_identity, b.execution_identity);
    assert_eq!(a.solution, b.solution);
    // No entitlement API exists: a service must authorize callers independently of this metadata.
    manifest(&mut f, |s| {
        s.replace("tier=\"paid\"", "tier=\"paid\" entitled=\"true\"")
    });
    assert!(load_package(&pack(&f, false, false)).is_err());
}

#[test]
fn changed_model_bytes_and_changed_inputs_recalculate_without_loose_xml_bypass() {
    let p = load_package(PACKAGE).unwrap();
    let mut f = files();
    let path = "models/maintenance-cost.xml";
    let changed = String::from_utf8(f[path].clone())
        .unwrap()
        .replace("field=\"price\"/>", "field=\"delay\"/>");
    asset(&mut f, path, changed.into_bytes());
    let q = load_package(&pack(&f, false, false)).unwrap();
    let a = p.compile(Some("cost"), INPUT).unwrap().solve().unwrap();
    let b = q.compile(Some("cost"), INPUT).unwrap().solve().unwrap();
    assert_ne!(a.execution_identity, b.execution_identity);
    assert_eq!(
        b.solution,
        p.compile(Some("delay"), INPUT)
            .unwrap()
            .solve()
            .unwrap()
            .solution
    );
    let mut data: Value = serde_json::from_str(INPUT).unwrap();
    for worker in data["crew"].as_array_mut().unwrap() {
        worker["active"] = json!(false);
    }
    assert_eq!(
        p.compile(None, &data.to_string())
            .unwrap()
            .solve()
            .unwrap()
            .solution
            .assigned,
        0
    );
    let mut f = files();
    let source = String::from_utf8(f[path].clone())
        .unwrap()
        .replace("value=\"1\"", "value=\"2\"");
    asset(&mut f, path, source.into_bytes());
    let b = load_package(&pack(&f, false, false))
        .unwrap()
        .compile(None, INPUT)
        .unwrap()
        .solve()
        .unwrap();
    assert_eq!(
        b.solution.assignments[0].resource_id.as_deref(),
        Some("mech-east")
    );
    assert_eq!(
        b.solution.assignments[1].resource_id.as_deref(),
        Some("mech-east")
    );
}

#[test]
fn cli_executes_archive_selected_entrypoint() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    for mode in ["cost", "delay"] {
        let p = std::process::Command::new(env!("CARGO_BIN_EXE_solve-package"))
            .arg(root.join("tests/fixtures/maintenance.orcpkg"))
            .arg(root.join("tests/fixtures/inputs/maintenance.json"))
            .arg(mode)
            .output()
            .unwrap();
        assert!(p.status.success(), "{}", String::from_utf8_lossy(&p.stderr));
        let output: Value = serde_json::from_slice(&p.stdout).unwrap();
        let expected: Value = serde_json::from_slice(
            &std::fs::read(root.join(format!("tests/fixtures/expected/{mode}.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(output["solution"], expected);
    }
}
