#[derive(Debug, serde::Serialize)]
pub struct DispatchRoute {
    pub courier_id: String,
    pub depot_departure_minute: u64,
    pub depot_return_minute: u64,
    pub visits: Vec<super::DispatchVisit>,
}
