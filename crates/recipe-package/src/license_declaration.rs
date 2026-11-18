use crate::PackagePath;
/// A publisher's license declaration. Parsing does not establish ownership or grant access.
#[derive(Debug, Clone)]
pub struct LicenseDeclaration {
    pub expression: String,
    pub terms: PackagePath,
}
