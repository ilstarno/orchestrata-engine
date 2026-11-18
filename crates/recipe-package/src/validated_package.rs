use crate::{
    AssetDeclaration, CompiledPackage, ContentIdentity, Entrypoint, PackageError, PackageMetadata,
    PackagePath, identity,
};
use orchestrata_recipe_runtime::mathematical::{compile_model, parse_model};
use std::collections::BTreeMap;
#[derive(Debug)]
pub struct ValidatedPackage {
    pub(crate) metadata: PackageMetadata,
    pub(crate) assets: BTreeMap<PackagePath, AssetDeclaration>,
    pub(crate) entrypoints: BTreeMap<String, Entrypoint>,
    pub(crate) default_entrypoint: String,
    pub(crate) files: BTreeMap<PackagePath, Vec<u8>>,
    pub(crate) identity: ContentIdentity,
}
impl ValidatedPackage {
    pub fn metadata(&self) -> &PackageMetadata {
        &self.metadata
    }
    pub fn assets(&self) -> impl Iterator<Item = &AssetDeclaration> {
        self.assets.values()
    }
    pub fn entrypoints(&self) -> impl Iterator<Item = &Entrypoint> {
        self.entrypoints.values()
    }
    pub fn default_entrypoint(&self) -> &str {
        &self.default_entrypoint
    }
    pub fn content_identity(&self) -> ContentIdentity {
        self.identity
    }
    pub fn terms_xml(&self) -> &str {
        std::str::from_utf8(&self.files[&self.metadata.license.terms]).expect("validated UTF-8")
    }
    /// Select only an explicitly declared mathematical model. No imports or filesystem access occur.
    pub fn compile(
        &self,
        selection: Option<&str>,
        records: &str,
    ) -> Result<CompiledPackage, PackageError> {
        let e = self
            .entrypoints
            .get(selection.unwrap_or(&self.default_entrypoint))
            .ok_or(PackageError::Entrypoint)?;
        let bytes = &self.files[&e.model];
        let xml = std::str::from_utf8(bytes).map_err(|_| PackageError::Xml)?;
        let spec = parse_model(xml).map_err(PackageError::Model)?;
        let model = compile_model(&spec, records).map_err(PackageError::Model)?;
        Ok(CompiledPackage {
            model,
            content_identity: self.identity,
            execution_identity: identity::execution(&e.id, &e.model, bytes),
        })
    }

    /// Explicit v2 penalty interface; never reinterprets a v1 model or result.
    pub fn compile_penalty(
        &self,
        selection: Option<&str>,
        records: &str,
    ) -> Result<crate::CompiledPenaltyPackage, PackageError> {
        use orchestrata_recipe_runtime::mathematical::{
            compile_penalty_model, parse_penalty_model,
        };
        let e = self
            .entrypoints
            .get(selection.unwrap_or(&self.default_entrypoint))
            .ok_or(PackageError::Entrypoint)?;
        let bytes = &self.files[&e.model];
        let xml = std::str::from_utf8(bytes).map_err(|_| PackageError::Xml)?;
        let model = compile_penalty_model(
            &parse_penalty_model(xml).map_err(PackageError::Model)?,
            records,
        )
        .map_err(PackageError::Model)?;
        Ok(crate::CompiledPenaltyPackage {
            model,
            content_identity: self.identity,
            execution_identity: identity::execution_penalty(&e.id, &e.model, bytes),
        })
    }

    /// Explicit routing interface with a caller-supplied total insertion budget.
    pub fn compile_route(
        &self,
        selection: Option<&str>,
        records: &str,
        budget: u32,
    ) -> Result<crate::CompiledRoutePackage, PackageError> {
        use orchestrata_recipe_runtime::route_dispatch::{compile_route_model, parse_route_model};
        let e = self
            .entrypoints
            .get(selection.unwrap_or(&self.default_entrypoint))
            .ok_or(PackageError::Entrypoint)?;
        let bytes = &self.files[&e.model];
        let xml = std::str::from_utf8(bytes).map_err(|_| PackageError::Xml)?;
        let model = compile_route_model(
            &parse_route_model(xml).map_err(|_| PackageError::Unsupported)?,
            records,
            budget,
        )
        .map_err(|_| PackageError::Unsupported)?;
        let execution_identity = identity::execution_route(&e.id, &e.model, bytes);
        let input_identity = identity::route_input(records.as_bytes());
        Ok(crate::CompiledRoutePackage {
            model,
            content_identity: self.identity,
            execution_identity,
            input_identity,
            request_identity: identity::route_request(
                self.identity,
                execution_identity,
                input_identity,
                budget,
            ),
        })
    }
}
