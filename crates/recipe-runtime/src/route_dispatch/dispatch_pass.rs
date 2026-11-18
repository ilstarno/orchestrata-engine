#[derive(Debug, serde::Serialize)]
pub struct DispatchPass {
    pub order: &'static str,
    pub evaluated_insertions: u32,
    pub stop: &'static str,
    pub selected: bool,
}
