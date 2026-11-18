use crate::{
    AssetDeclaration, AssetKind, DistributionTier, Entrypoint, LicenseDeclaration, LocalizedText,
    PackageError, PackageMetadata, PackagePath, PackageVersion, limits::*, manifest::Manifest,
    xml::*,
};
use std::collections::BTreeMap;
pub(crate) fn parse(xml: &str) -> Result<Manifest, PackageError> {
    let doc = document(xml)?;
    let root = doc.root_element();
    tag(
        root,
        "recipe-package",
        &[
            "format-version",
            "id",
            "version",
            "default-locale",
            "default-entrypoint",
        ],
    )?;
    if attr(root, "format-version")? != "1" {
        return Err(PackageError::Unsupported);
    }
    let id = attr(root, "id")?;
    let default_locale = attr(root, "default-locale")?;
    let default = attr(root, "default-entrypoint")?;
    if !identifier(id) || !identifier(default) || !locale(default_locale) {
        return Err(PackageError::Metadata);
    }
    let version = PackageVersion::parse(attr(root, "version")?)?;
    let [
        author,
        license,
        distribution,
        translations,
        assets,
        entrypoints,
    ] = children(root)?.try_into().map_err(|_| PackageError::Xml)?;
    let author = text(author, "author", 160)?;
    leaf(license, "license", &["expression", "terms"])?;
    let expression = attr(license, "expression")?;
    if expression.len() > 256
        || expression.trim() != expression
        || spdx::Expression::parse(expression).is_err()
    {
        return Err(PackageError::Metadata);
    }
    let license = LicenseDeclaration {
        expression: expression.to_owned(),
        terms: PackagePath::parse(attr(license, "terms")?)?,
    };
    leaf(distribution, "distribution", &["tier"])?;
    let distribution = match attr(distribution, "tier")? {
        "free" => DistributionTier::Free,
        "paid" => DistributionTier::Paid,
        _ => return Err(PackageError::Metadata),
    };
    tag(translations, "translations", &[])?;
    let mut localized = BTreeMap::new();
    for t in children(translations)? {
        tag(t, "translation", &["locale"])?;
        let language = attr(t, "locale")?;
        if !locale(language) || localized.len() >= 8 {
            return Err(PackageError::Metadata);
        }
        let [title, description] = children(t)?.try_into().map_err(|_| PackageError::Xml)?;
        let value = LocalizedText {
            title: text(title, "title", 200)?,
            description: text(description, "description", 2000)?,
        };
        if localized.insert(language.to_owned(), value).is_some() {
            return Err(PackageError::Metadata);
        }
    }
    if !localized.contains_key("en")
        || !localized.contains_key("sq")
        || !localized.contains_key(default_locale)
    {
        return Err(PackageError::Metadata);
    }
    tag(assets, "assets", &[])?;
    let mut inventory = BTreeMap::new();
    for a in children(assets)? {
        leaf(a, "asset", &["path", "kind", "bytes", "sha256"])?;
        let path = PackagePath::parse(attr(a, "path")?)?;
        let kind = match attr(a, "kind")? {
            "model" => AssetKind::Model,
            "terms" => AssetKind::Terms,
            _ => return Err(PackageError::Unsupported),
        };
        let bytes = attr(a, "bytes")?
            .parse::<usize>()
            .map_err(|_| PackageError::Metadata)?;
        if bytes == 0
            || bytes > MAX_MEMBER_BYTES
            || path.as_str() == "recipe.xml"
            || inventory.len() >= MAX_MEMBERS - 1
        {
            return Err(PackageError::Inventory);
        }
        let h = attr(a, "sha256")?;
        if h.len() != 64
            || !h
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        {
            return Err(PackageError::Digest);
        }
        let mut hash = [0; 32];
        for (i, b) in hash.iter_mut().enumerate() {
            *b = u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|_| PackageError::Digest)?;
        }
        if inventory
            .insert(
                path.clone(),
                AssetDeclaration {
                    path,
                    kind,
                    bytes,
                    sha256: hash,
                },
            )
            .is_some()
        {
            return Err(PackageError::Inventory);
        }
    }
    if inventory.get(&license.terms).map(|a| &a.kind) != Some(&AssetKind::Terms) {
        return Err(PackageError::Inventory);
    }
    tag(entrypoints, "entrypoints", &[])?;
    let mut entries = BTreeMap::new();
    for e in children(entrypoints)? {
        leaf(e, "entrypoint", &["id", "model"])?;
        let id = attr(e, "id")?;
        let path = PackagePath::parse(attr(e, "model")?)?;
        if !identifier(id)
            || entries.len() >= 8
            || inventory.get(&path).map(|a| &a.kind) != Some(&AssetKind::Model)
        {
            return Err(PackageError::Entrypoint);
        }
        if entries
            .insert(
                id.to_owned(),
                Entrypoint {
                    id: id.to_owned(),
                    model: path,
                },
            )
            .is_some()
        {
            return Err(PackageError::Entrypoint);
        }
    }
    if !entries.contains_key(default)
        || inventory
            .values()
            .any(|a| a.kind == AssetKind::Model && !entries.values().any(|e| e.model == a.path))
    {
        return Err(PackageError::Entrypoint);
    }
    Ok(Manifest {
        metadata: PackageMetadata {
            id: id.to_owned(),
            version,
            author,
            default_locale: default_locale.to_owned(),
            translations: localized,
            license,
            distribution,
        },
        assets: inventory,
        entrypoints: entries,
        default_entrypoint: default.to_owned(),
    })
}
