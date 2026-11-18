use crate::{
    AssetKind, PackageError, PackagePath, ValidatedPackage, identity, parse_manifest, read_archive,
    xml,
};
use orchestrata_recipe_runtime::mathematical::{parse_model, parse_penalty_model};
use sha2::{Digest, Sha256};
/// Validate the complete archive and every declared asset before exposing any metadata or entrypoint.
pub fn load_package(bytes: &[u8]) -> Result<ValidatedPackage, PackageError> {
    let files = read_archive::read(bytes)?;
    let root = PackagePath::parse("recipe.xml")?;
    let manifest_bytes = files.get(&root).ok_or(PackageError::Inventory)?;
    let manifest =
        parse_manifest::parse(std::str::from_utf8(manifest_bytes).map_err(|_| PackageError::Xml)?)?;
    if files.len() != manifest.assets.len() + 1 {
        return Err(PackageError::Inventory);
    }
    for a in manifest.assets.values() {
        let bytes = files.get(&a.path).ok_or(PackageError::Inventory)?;
        if bytes.len() != a.bytes {
            return Err(PackageError::Inventory);
        }
        let hash: [u8; 32] = Sha256::digest(bytes).into();
        if hash != a.sha256 {
            return Err(PackageError::Digest);
        }
        let text = std::str::from_utf8(bytes).map_err(|_| PackageError::Xml)?;
        let doc = xml::document(text)?;
        match a.kind {
            AssetKind::Model if doc.root_element().has_tag_name("route-dispatch") => {
                orchestrata_recipe_runtime::route_dispatch::parse_route_model(text)
                    .map_err(|_| PackageError::Unsupported)?;
            }
            AssetKind::Model => match doc.root_element().attribute("version") {
                Some("1") => {
                    parse_model(text).map_err(|_| PackageError::Unsupported)?;
                }
                Some("2") => {
                    parse_penalty_model(text).map_err(|_| PackageError::Unsupported)?;
                }
                _ => return Err(PackageError::Unsupported),
            },
            AssetKind::Terms => {
                let root = doc.root_element();
                xml::tag(root, "terms", &["version"])?;
                if xml::attr(root, "version")? != "1" {
                    return Err(PackageError::Unsupported);
                }
                let mut locales = std::collections::BTreeSet::new();
                for n in xml::children(root)? {
                    xml::tag(n, "text", &["locale"])?;
                    let language = xml::attr(n, "locale")?;
                    let s = n.text().unwrap_or_default();
                    if !xml::locale(language)
                        || !locales.insert(language)
                        || n.children().any(|c| !c.is_text())
                        || s.trim().is_empty()
                        || s.len() > 16000
                        || s.chars().any(|c| c.is_control() && !"\n\r\t".contains(c))
                    {
                        return Err(PackageError::Metadata);
                    }
                }
                if !locales.contains(manifest.metadata.default_locale.as_str()) {
                    return Err(PackageError::Metadata);
                }
            }
        }
    }
    let identity = identity::content(&files);
    Ok(ValidatedPackage {
        metadata: manifest.metadata,
        assets: manifest.assets,
        entrypoints: manifest.entrypoints,
        default_entrypoint: manifest.default_entrypoint,
        files,
        identity,
    })
}
