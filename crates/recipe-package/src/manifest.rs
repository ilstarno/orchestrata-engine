use crate::{AssetDeclaration, Entrypoint, PackageMetadata, PackagePath};
use std::collections::BTreeMap;
#[derive(Debug)]
pub(crate) struct Manifest {
    pub metadata: PackageMetadata,
    pub assets: BTreeMap<PackagePath, AssetDeclaration>,
    pub entrypoints: BTreeMap<String, Entrypoint>,
    pub default_entrypoint: String,
}
