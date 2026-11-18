#[derive(Debug, serde::Serialize)]
pub struct DispatchSolution {
    pub schema: &'static str,
    pub algorithm: &'static str,
    pub guarantee: &'static str,
    pub time_unit: &'static str,
    pub load_unit: &'static str,
    pub supplied_max_insertions: u32,
    pub evaluated_insertions: u32,
    pub total_courier_minutes: u64,
    pub routes: Vec<super::DispatchRoute>,
    pub unassigned: Vec<String>,
    pub passes: Vec<super::DispatchPass>,
}
