use crate::{ContentIdentity, ExecutionIdentity, PackagePath};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
fn field(h: &mut Sha256, bytes: &[u8]) {
    h.update((bytes.len() as u64).to_be_bytes());
    h.update(bytes);
}
pub(crate) fn content(files: &BTreeMap<PackagePath, Vec<u8>>) -> ContentIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-content-v1\0");
    h.update((files.len() as u64).to_be_bytes());
    for (path, bytes) in files {
        field(&mut h, path.as_str().as_bytes());
        field(&mut h, bytes);
    }
    ContentIdentity(h.finalize().into())
}
pub(crate) fn execution(id: &str, path: &PackagePath, bytes: &[u8]) -> ExecutionIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-execution-math-v1\0");
    field(&mut h, id.as_bytes());
    field(&mut h, path.as_str().as_bytes());
    field(&mut h, bytes);
    ExecutionIdentity(h.finalize().into())
}
