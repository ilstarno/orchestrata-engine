use crate::{AssetKind, PackagePath};
#[derive(Debug, Clone)]
pub struct AssetDeclaration {
    pub path: PackagePath,
    pub kind: AssetKind,
    pub bytes: usize,
    pub sha256: [u8; 32],
}
