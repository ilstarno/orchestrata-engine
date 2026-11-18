#[derive(Debug, serde::Serialize)]
pub struct DispatchVisit {
    pub shipment_id: String,
    pub kind: &'static str,
    pub location: String,
    pub arrival_minute: u64,
    pub service_start_minute: u64,
    pub service_end_minute: u64,
}
