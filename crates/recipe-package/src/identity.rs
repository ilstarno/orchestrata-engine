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

pub(crate) fn execution_penalty(id: &str, path: &PackagePath, bytes: &[u8]) -> ExecutionIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-execution-math-v2-penalty\0");
    field(&mut h, id.as_bytes());
    field(&mut h, path.as_str().as_bytes());
    field(&mut h, bytes);
    ExecutionIdentity(h.finalize().into())
}

pub(crate) fn execution_route(id: &str, path: &PackagePath, bytes: &[u8]) -> ExecutionIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-execution-route-dispatch-v1\0");
    field(&mut h, id.as_bytes());
    field(&mut h, path.as_str().as_bytes());
    field(&mut h, bytes);
    ExecutionIdentity(h.finalize().into())
}
pub(crate) fn route_input(bytes: &[u8]) -> crate::InputIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-route-input-v1\0");
    field(&mut h, bytes);
    crate::InputIdentity(h.finalize().into())
}
pub(crate) fn route_request(
    content: ContentIdentity,
    execution: ExecutionIdentity,
    input: crate::InputIdentity,
    budget: u32,
) -> crate::RequestIdentity {
    let mut h = Sha256::new();
    h.update(b"orcpkg-route-request-v1\0");
    h.update(content.0);
    h.update(execution.0);
    h.update(input.0);
    h.update(budget.to_be_bytes());
    crate::RequestIdentity(h.finalize().into())
}
