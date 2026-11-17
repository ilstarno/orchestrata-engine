use super::LocationId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteVerificationError {
    InvalidShift(usize),
    InvalidShipment(usize),
    UnknownCourier(usize),
    DuplicateCourierRoute(usize),
    UnknownShipment(usize),
    ForbiddenMode(usize),
    DuplicatePickup(usize),
    DuplicateDelivery(usize),
    DeliveryBeforePickup(usize),
    WrongDeliveryCourier(usize),
    MissingPickup(usize),
    MissingDelivery(usize),
    UnreachableLeg {
        courier: usize,
        from: LocationId,
        to: LocationId,
    },
    ArrivalTooEarly(usize),
    StopWindowViolation(usize),
    ShiftViolation(usize),
    DepotReturnTooEarly(usize),
    CapacityExceeded(usize),
    ArithmeticOverflow,
}
