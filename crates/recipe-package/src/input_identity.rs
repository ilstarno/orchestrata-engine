#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputIdentity(pub(crate) [u8; 32]);
impl InputIdentity {
    pub fn to_hex(self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}
