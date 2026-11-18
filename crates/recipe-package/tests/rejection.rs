mod common;
use common::*;
use orchestrata_recipe_package::{PackageError, PackagePath, limits::*, load_package};
use std::collections::BTreeMap;

#[test]
fn entire_inventory_is_checked_before_any_entrypoint_is_exposed() {
    let mut f = files();
    f.insert("extra.xml".into(), b"<extra/>".to_vec());
    assert!(load_package(&pack(&f, false, false)).is_err());
    let mut f = files();
    f.remove("legal/terms.xml");
    assert!(load_package(&pack(&f, false, false)).is_err());
    let mut f = files();
    f.insert("models/maintenance-delay.xml".into(), b"<model/>".to_vec());
    assert!(load_package(&pack(&f, false, false)).is_err());
    let mut f = files();
    let path = "models/maintenance-delay.xml";
    let source = String::from_utf8(f[path].clone())
        .unwrap()
        .replace("edge-sum", "unsupported-objective");
    asset(&mut f, path, source.into_bytes());
    assert!(matches!(
        load_package(&pack(&f, false, false)),
        Err(PackageError::Unsupported)
    ));
    let mut f = files();
    let source = String::from_utf8(f[path].clone())
        .unwrap()
        .replace("maintenance", "maintenancf");
    f.insert(path.into(), source.into_bytes());
    assert!(matches!(
        load_package(&pack(&f, false, false)),
        Err(PackageError::Digest)
    ));
}

#[test]
fn malformed_metadata_or_undeclared_execution_is_rejected() {
    for (old, new) in [
        ("format-version=\"1\"", "format-version=\"2\""),
        ("version=\"1.0.0\"", "version=\"1.0.01\""),
        (
            "default-entrypoint=\"cost\"",
            "default-entrypoint=\"missing\"",
        ),
        ("locale=\"sq\"", "locale=\"fr\""),
        ("locale=\"sq\"", "locale=\"en\""),
        ("Synthetic test fixture (publisher unspecified)", ""),
        ("expression=\"Apache-2.0\"", "expression=\"NOT-A-LICENSE\""),
        ("tier=\"free\"", "tier=\"licensed\""),
        (
            "terms=\"legal/terms.xml\"",
            "terms=\"https://example.invalid/terms.xml\"",
        ),
        ("id=\"delay\" model=", "id=\"cost\" model="),
        ("kind=\"terms\"", "kind=\"program\""),
        (
            "</entrypoints>",
            "<entrypoint id=\"terms\" model=\"legal/terms.xml\"/></entrypoints>",
        ),
    ] {
        let mut f = files();
        manifest(&mut f, |s| {
            assert!(s.contains(old));
            s.replace(old, new)
        });
        assert!(
            load_package(&pack(&f, false, false)).is_err(),
            "accepted {old}"
        );
    }
}

#[test]
fn unsafe_xml_or_legacy_descriptive_payload_is_not_promoted_to_math() {
    for source in [
        b"<!DOCTYPE model [<!ENTITY a SYSTEM 'https://example.invalid/x'>]><model/>".to_vec(),
        b"<?execute x?><model/>".to_vec(),
        b"<recipe version=\"1\"/>".to_vec(),
        b"<assignment-recipe version=\"2\"/>".to_vec(),
        b"<model xmlns=\"urn:x\"/>".to_vec(),
    ] {
        let mut f = files();
        asset(&mut f, "models/maintenance-delay.xml", source);
        assert!(load_package(&pack(&f, false, false)).is_err());
    }
    let mut f = files();
    manifest(&mut f, |s| format!("<!DOCTYPE recipe-package>{s}"));
    assert!(load_package(&pack(&f, false, false)).is_err());
    let mut f = files();
    asset(
        &mut f,
        "legal/terms.xml",
        b"<terms version=\"1\"><include href=\"https://example.invalid/terms.xml\"/></terms>"
            .to_vec(),
    );
    assert!(load_package(&pack(&f, false, false)).is_err());
}

#[test]
fn canonical_paths_reject_traversal_aliases_devices_and_directories() {
    for path in [
        "../a.xml",
        "/a.xml",
        "a/../b.xml",
        "a//b.xml",
        "a\\b.xml",
        "A.xml",
        "models/.x.xml",
        "aux.xml",
        "a.xml/",
        "a:foo.xml",
        "x%2e.xml",
        "ä.xml",
        "models/cost.xml/../delay.xml",
    ] {
        assert!(PackagePath::parse(path).is_err(), "accepted {path}");
        let mut f = files();
        f.insert(path.into(), b"<x/>".to_vec());
        assert!(load_package(&pack(&f, false, false)).is_err());
    }
    let mut f = files();
    f.insert("Recipe.xml".into(), f["recipe.xml"].clone());
    assert!(load_package(&pack(&f, false, false)).is_err());
}

#[test]
fn duplicates_symlinks_encryption_unknown_compression_and_header_mismatches_fail() {
    let map = BTreeMap::from([
        ("a.xml".into(), b"<a/>".to_vec()),
        ("b.xml".into(), b"<b/>".to_vec()),
    ]);
    let bytes = pack(&map, false, true);
    let positions = central_positions(&bytes);
    let mut duplicate = bytes.clone();
    duplicate[positions[1] + 46] = b'a';
    assert!(matches!(
        load_package(&duplicate),
        Err(PackageError::Inventory)
    ));
    let bytes = pack(&files(), false, true);
    let p = central_positions(&bytes)[0];
    for kind in 0..5 {
        let mut b = bytes.clone();
        match kind {
            0 => {
                b[p + 5] = 3;
                b[p + 38..p + 42].copy_from_slice(&(0o120777u32 << 16).to_le_bytes());
            }
            1 => b[p + 8] |= 1,
            2 => b[p + 10..p + 12].copy_from_slice(&12u16.to_le_bytes()),
            3 => b[30] ^= 1,
            _ => {
                let second = central_positions(&b)[1];
                b[second + 42..second + 46].copy_from_slice(&0u32.to_le_bytes());
            }
        }
        assert!(load_package(&b).is_err());
    }
}

#[test]
fn archive_member_count_declared_and_actual_expansion_are_bounded() {
    assert!(matches!(
        load_package(&vec![0; MAX_ARCHIVE_BYTES + 1]),
        Err(PackageError::Limit)
    ));
    let map = (0..17)
        .map(|i| (format!("m{i}.xml"), b"<x/>".to_vec()))
        .collect();
    assert!(matches!(
        load_package(&pack(&map, false, false)),
        Err(PackageError::Limit)
    ));
    let map = BTreeMap::from([("recipe.xml".into(), vec![b' '; MAX_MEMBER_BYTES + 1])]);
    let b = pack(&map, false, false);
    assert!(matches!(load_package(&b), Err(PackageError::Limit)));
    // Lie consistently in both size headers; bounded reading must still reject expansion.
    let mut lied = b.clone();
    let p = central_positions(&lied)[0];
    lied[22..26].copy_from_slice(&1u32.to_le_bytes());
    lied[p + 24..p + 28].copy_from_slice(&1u32.to_le_bytes());
    assert!(load_package(&lied).is_err());
    let map = (0..9)
        .map(|i| (format!("m{i}.xml"), vec![b' '; MAX_MEMBER_BYTES]))
        .collect();
    assert!(matches!(
        load_package(&pack(&map, false, false)),
        Err(PackageError::Limit)
    ));
}

#[test]
fn truncation_corruption_and_archive_suffix_never_panic_or_return_partial_package() {
    for end in 0..PACKAGE.len() {
        assert!(
            std::panic::catch_unwind(|| load_package(&PACKAGE[..end]))
                .unwrap()
                .is_err()
        );
    }
    let mut appended = PACKAGE.to_vec();
    appended.extend_from_slice(b"trailing");
    assert!(load_package(&appended).is_err());
    let b = pack(&files(), false, true);
    let mut corrupt = b.clone();
    corrupt[45] ^= 1;
    assert!(load_package(&corrupt).is_err());
    for index in (0..PACKAGE.len()).step_by(17) {
        let mut b = PACKAGE.to_vec();
        b[index] ^= 0x40;
        let _ = std::panic::catch_unwind(|| load_package(&b)).expect("package parser panicked");
    }
}
