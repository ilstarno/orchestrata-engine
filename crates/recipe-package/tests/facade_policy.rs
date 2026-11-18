use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{Attribute, Item, Meta, Token, punctuated::Punctuated};

fn metadata_is_wiring(meta: &Meta) -> bool {
    let path = meta.path();
    if path.is_ident("cfg_attr") {
        let Meta::List(list) = meta else {
            return false;
        };
        let Ok(nested) = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        else {
            return false;
        };
        return nested.len() >= 2 && nested.iter().skip(1).all(metadata_is_wiring);
    }
    [
        "cfg", "doc", "path", "allow", "warn", "deny", "forbid", "expect",
    ]
    .iter()
    .any(|name| path.is_ident(name))
}

fn attributes_are_wiring(attributes: &[Attribute]) -> bool {
    attributes.iter().all(|a| metadata_is_wiring(&a.meta))
}

fn is_wiring(source: &str) -> bool {
    let Ok(file) = syn::parse_file(source) else {
        return false;
    };
    attributes_are_wiring(&file.attrs)
        && file.items.iter().all(|item| match item {
            Item::Mod(module) => module.content.is_none() && attributes_are_wiring(&module.attrs),
            Item::Use(import) => attributes_are_wiring(&import.attrs),
            Item::ExternCrate(import) => attributes_are_wiring(&import.attrs),
            _ => false,
        })
}

fn facade(path: &Path) -> bool {
    path.file_name().is_some_and(|name| {
        ["lib.rs", "mod.rs", "module.rs"]
            .iter()
            .any(|candidate| name == *candidate)
    })
}

fn tracked_facades(root: &Path) -> Option<Vec<PathBuf>> {
    let top = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(root)
        .output()
        .ok()?;
    if !top.status.success()
        || Path::new(std::str::from_utf8(&top.stdout).ok()?.trim())
            .canonicalize()
            .ok()?
            != root.canonicalize().ok()?
    {
        return None;
    }
    let output = Command::new("git")
        .args(["ls-files", "--cached", "-z"])
        .current_dir(root)
        .output()
        .expect("read tracked source inventory");
    assert!(
        output.status.success(),
        "cannot read tracked source inventory"
    );
    let mut paths = output
        .stdout
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .map(|p| PathBuf::from(std::str::from_utf8(p).expect("UTF-8 source path")))
        .filter(|p| facade(p))
        .collect::<Vec<_>>();
    paths.sort();
    paths.dedup();
    Some(paths)
}

fn collect_facades(root: &Path, directory: &Path, paths: &mut Vec<PathBuf>) {
    if !directory.exists() {
        return;
    }
    for entry in fs::read_dir(directory).unwrap() {
        let entry = entry.unwrap();
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            if entry.file_name() != "target" && entry.file_name() != ".git" {
                collect_facades(root, &entry.path(), paths);
            }
        } else if facade(&entry.path()) {
            assert!(kind.is_file(), "non-regular facade: {:?}", entry.path());
            paths.push(entry.path().strip_prefix(root).unwrap().to_owned());
        }
    }
}

fn archive_facades(root: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for name in [
        "ffi",
        "solver",
        "recipe-runtime",
        "recipe-catalog",
        "recipe-package",
    ] {
        for tree in ["src", "tests"] {
            collect_facades(root, &root.join("crates").join(name).join(tree), &mut paths);
        }
    }
    for tree in [
        "backend_tests/rust",
        "examples/browser/src",
        "examples/assignment-runner/src",
        "examples/assignment-runner/tests",
    ] {
        collect_facades(root, &root.join(tree), &mut paths);
    }
    paths.sort();
    paths
}

fn file_is_wiring(path: &Path) -> bool {
    assert!(
        fs::symlink_metadata(path).unwrap().file_type().is_file(),
        "non-regular facade: {path:?}"
    );
    is_wiring(&fs::read_to_string(path).unwrap())
}

#[test]
fn repository_facades_only_wire_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let paths = tracked_facades(&root).unwrap_or_else(|| archive_facades(&root));
    assert!(!paths.is_empty());
    for path in paths {
        assert!(
            file_is_wiring(&root.join(&path)),
            "implementation in {path:?}"
        );
    }
}

#[test]
fn source_archive_fallback_covers_tracked_inventory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    if let Some(paths) = tracked_facades(&root) {
        let fallback = archive_facades(&root);
        assert!(paths.iter().all(|path| fallback.contains(path)));
    }
}

#[test]
fn ignores_untracked_checkout_but_checks_owned_archive_source() {
    let root = std::env::temp_dir().join(format!("orchestrata-facade-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    let source = root.join("crates/solver/src");
    let scratch = root.join("extra-checkout/src");
    fs::create_dir_all(&source).unwrap();
    fs::create_dir_all(&scratch).unwrap();
    fs::write(source.join("lib.rs"), "pub mod implementation;").unwrap();
    fs::write(
        scratch.join("lib.rs"),
        "pub fn not_owned() {} impl Example {}",
    )
    .unwrap();
    for hidden in [
        "backend_tests/rust/target/generated",
        "crates/solver/src/.git/generated",
    ] {
        let directory = root.join(hidden);
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("mod.rs"), "pub fn generated() {}").unwrap();
    }
    assert!(tracked_facades(&root).is_none());
    let owned = vec![PathBuf::from("crates/solver/src/lib.rs")];
    assert_eq!(archive_facades(&root), owned);
    assert!(
        Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["add", "crates/solver/src/lib.rs"])
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(tracked_facades(&root), Some(owned));
    let bad = "backend_tests/rust/target/generated/mod.rs";
    assert!(
        Command::new("git")
            .args(["add", "--force", bad])
            .current_dir(&root)
            .status()
            .unwrap()
            .success()
    );
    let tracked = tracked_facades(&root).unwrap();
    assert!(tracked.contains(&PathBuf::from(bad)));
    assert!(tracked.iter().any(|path| !file_is_wiring(&root.join(path))));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn accepts_module_wiring_and_documentation() {
    assert!(is_wiring(
        "//! API docs\n#![allow(dead_code)]\n#[cfg(test)]\n#[path=\"checks.rs\"]\nmod checks;\npub mod api;\npub use api::{Result, run};\nextern crate core;"
    ));
    assert!(is_wiring("#[cfg_attr(test, allow(dead_code))] mod checks;"));
}

#[test]
fn rejects_implementation_even_when_disabled_or_hidden_by_macros() {
    for source in [
        "fn run() {}",
        "impl Example { fn run(&self) {} }",
        "#[cfg(any())] fn disabled() {}",
        "pub const LIMIT: usize = 10;",
        "static LIMIT: usize = 10;",
        "struct Example;",
        "enum Example { One }",
        "trait Example {}",
        "type Example = u32;",
        "mod nested { pub fn run() {} }",
        "mod nested {}",
        "include!(\"implementation.rs\");",
        "macro_rules! implement { () => { fn run() {} }; }",
        "#[generate_code] mod generated;",
        "#[cfg_attr(test, generate_code)] mod generated;",
        "fn broken(",
    ] {
        assert!(!is_wiring(source), "accepted implementation: {source}");
    }
}
