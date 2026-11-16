/// Maximum number of couriers accepted by the small route constructor.
pub const MAX_CONSTRUCTION_COURIERS: usize = 16;
/// Maximum number of shipments accepted by the small route constructor.
pub const MAX_CONSTRUCTION_SHIPMENTS: usize = 32;
/// Maximum caller-supplied budget for candidate route simulations.
pub const MAX_CONSTRUCTION_INSERTIONS: u32 = 20_000;
pub(super) const DISTINCT_COURIER_MODES: usize = 3;
