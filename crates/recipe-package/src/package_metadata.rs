use crate::{DistributionTier, LicenseDeclaration, LocalizedText, PackageVersion};
use std::collections::BTreeMap;
#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub id: String,
    pub version: PackageVersion,
    pub author: String,
    pub default_locale: String,
    pub translations: BTreeMap<String, LocalizedText>,
    pub license: LicenseDeclaration,
    pub distribution: DistributionTier,
}
