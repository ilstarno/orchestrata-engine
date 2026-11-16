#[derive(Clone, Copy)]
pub(super) enum ShipmentStatus {
    Pending,
    Onboard(usize),
    Delivered,
}
