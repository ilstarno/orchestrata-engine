use crate::PackagePath;
#[derive(Debug, Clone)]
pub struct Entrypoint {
    pub id: String,
    pub model: PackagePath,
}
