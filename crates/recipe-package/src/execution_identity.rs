#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionIdentity(pub(crate) [u8; 32]);
impl ExecutionIdentity {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
    pub fn to_hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}
