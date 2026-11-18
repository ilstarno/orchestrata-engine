use std::{error::Error, fmt};
#[derive(Debug, PartialEq, Eq)]
pub enum PackageError {
    Archive,
    Limit,
    Path,
    Inventory,
    Digest,
    Xml,
    Metadata,
    Unsupported,
    Entrypoint,
    Model(orchestrata_recipe_runtime::mathematical::ModelError),
}
impl fmt::Display for PackageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::Model(error) = self {
            return error.fmt(f);
        }
        f.write_str(match self {
            Self::Archive => "Invalid or unsupported package ZIP structure.",
            Self::Limit => "Package exceeds a supported resource bound.",
            Self::Path => "Invalid package member path.",
            Self::Inventory => "Package inventory does not match its manifest.",
            Self::Digest => "Package member digest does not match its declaration.",
            Self::Xml => "Invalid or unsafe package XML.",
            Self::Metadata => "Invalid package metadata.",
            Self::Unsupported => "Unsupported package content or semantics.",
            Self::Entrypoint => "No declared model entrypoint matches the selection.",
            Self::Model(_) => unreachable!("handled above"),
        })
    }
}
impl Error for PackageError {}
