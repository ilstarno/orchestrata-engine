use super::{
    Courier, CourierMode, LocationId, RouteVerificationError as Error, RouteVisit, Shipment, Stop,
    VisitKind, shipment_status::ShipmentStatus,
};

pub(super) struct RouteCursor {
    pub location: LocationId,
    pub departure: u64,
    pub load: u64,
}

impl RouteCursor {
    pub fn visit(
        &mut self,
        courier_index: usize,
        courier: &Courier,
        visit: &RouteVisit,
        shipment: &Shipment,
        status: &mut ShipmentStatus,
        travel: &mut impl FnMut(CourierMode, LocationId, LocationId, u64) -> Option<u64>,
    ) -> Result<(), Error> {
        if !shipment.allowed_modes.contains(&courier.mode) {
            return Err(Error::ForbiddenMode(visit.shipment));
        }

        let stop = Self::stop(visit, shipment, *status, courier_index)?;
        let earliest = self.earliest_arrival(courier_index, courier.mode, stop.location, travel)?;
        if visit.arrival < earliest {
            return Err(Error::ArrivalTooEarly(visit.shipment));
        }

        let service_start = visit.arrival.max(stop.window.start);
        let finish = service_start
            .checked_add(stop.service_time)
            .ok_or(Error::ArithmeticOverflow)?;
        if finish > stop.window.end {
            return Err(Error::StopWindowViolation(visit.shipment));
        }
        if finish > courier.shift.end {
            return Err(Error::ShiftViolation(courier_index));
        }

        match visit.kind {
            VisitKind::Pickup => {
                self.load = self
                    .load
                    .checked_add(shipment.weight)
                    .filter(|&load| load <= courier.capacity)
                    .ok_or(Error::CapacityExceeded(visit.shipment))?;
                *status = ShipmentStatus::Onboard(courier_index);
            }
            VisitKind::Delivery => {
                self.load = self
                    .load
                    .checked_sub(shipment.weight)
                    .ok_or(Error::ArithmeticOverflow)?;
                *status = ShipmentStatus::Delivered;
            }
        }

        self.location = stop.location;
        self.departure = finish;
        Ok(())
    }

    pub fn earliest_arrival(
        &self,
        courier: usize,
        mode: CourierMode,
        destination: LocationId,
        travel: &mut impl FnMut(CourierMode, LocationId, LocationId, u64) -> Option<u64>,
    ) -> Result<u64, Error> {
        if self.location == destination {
            return Ok(self.departure);
        }

        let length = travel(mode, self.location, destination, self.departure).ok_or(
            Error::UnreachableLeg {
                courier,
                from: self.location,
                to: destination,
            },
        )?;
        self.departure
            .checked_add(length)
            .ok_or(Error::ArithmeticOverflow)
    }

    fn stop(
        visit: &RouteVisit,
        shipment: &Shipment,
        status: ShipmentStatus,
        courier: usize,
    ) -> Result<Stop, Error> {
        match (visit.kind, status) {
            (VisitKind::Pickup, ShipmentStatus::Pending) => Ok(shipment.pickup),
            (VisitKind::Pickup, _) => Err(Error::DuplicatePickup(visit.shipment)),
            (VisitKind::Delivery, ShipmentStatus::Pending) => {
                Err(Error::DeliveryBeforePickup(visit.shipment))
            }
            (VisitKind::Delivery, ShipmentStatus::Delivered) => {
                Err(Error::DuplicateDelivery(visit.shipment))
            }
            (VisitKind::Delivery, ShipmentStatus::Onboard(owner)) if owner == courier => {
                Ok(shipment.delivery)
            }
            (VisitKind::Delivery, ShipmentStatus::Onboard(_)) => {
                Err(Error::WrongDeliveryCourier(visit.shipment))
            }
        }
    }
}
