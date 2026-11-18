use super::{snapshot::Snapshot, stop_row::StopRow};
use orchestrata_solver::routing::{
    Courier, CourierMode, LocationId, RoutingProblem, Shipment, Stop,
};
use std::collections::{HashMap, HashSet};
pub(super) type Travel = HashMap<(CourierMode, LocationId, LocationId), u64>;
pub fn compile(s: &Snapshot) -> Result<(RoutingProblem, Travel), String> {
    if s.locations.len() > 128
        || s.couriers.len() > 16
        || s.shipments.len() > 32
        || s.travel_legs.len() > 8192
    {
        return Err("route input exceeds limits".into());
    }
    let bounded_window = |w: &super::window::Window| w.start <= w.end && w.end <= 1_000_000;
    if s.couriers
        .iter()
        .any(|c| c.capacity_kg > 1_000_000 || !bounded_window(&c.shift))
        || s.shipments.iter().any(|j| {
            j.weight_kg > 1_000_000
                || j.allowed_modes.len() > 3
                || !bounded_window(&j.pickup.window)
                || !bounded_window(&j.delivery.window)
                || j.pickup.service_minutes > 1_000_000
                || j.delivery.service_minutes > 1_000_000
        })
        || s.travel_legs.iter().any(|leg| leg.minutes > 1_000_000)
    {
        return Err("route numeric bounds or windows invalid".into());
    }
    unique(s.locations.iter().map(String::as_str))?;
    unique(s.couriers.iter().map(|r| r.id.as_str()))?;
    unique(s.shipments.iter().map(|r| r.id.as_str()))?;
    let locations = s
        .locations
        .iter()
        .enumerate()
        .map(|(i, id)| (id.as_str(), LocationId(i as u32)))
        .collect::<HashMap<_, _>>();
    let lookup = |key: &str| {
        locations
            .get(key)
            .copied()
            .ok_or_else(|| "unknown location".to_owned())
    };
    let stop = |row: &StopRow| -> Result<Stop, String> {
        Ok(Stop {
            location: lookup(&row.location)?,
            window: row.window.into(),
            service_time: row.service_minutes,
        })
    };
    let couriers = s
        .couriers
        .iter()
        .map(|r| -> Result<_, String> {
            Ok(Courier {
                mode: r.mode.into(),
                depot: lookup(&r.depot)?,
                shift: r.shift.into(),
                capacity: r.capacity_kg,
            })
        })
        .collect::<Result<_, _>>()?;
    let shipments = s
        .shipments
        .iter()
        .map(|r| -> Result<_, String> {
            Ok(Shipment {
                pickup: stop(&r.pickup)?,
                delivery: stop(&r.delivery)?,
                weight: r.weight_kg,
                allowed_modes: r.allowed_modes.iter().copied().map(Into::into).collect(),
            })
        })
        .collect::<Result<_, _>>()?;
    let mut travel = Travel::new();
    for row in &s.travel_legs {
        let pair = (row.mode.into(), lookup(&row.from)?, lookup(&row.to)?);
        if pair.1 == pair.2 || travel.insert(pair, row.minutes).is_some() {
            return Err("directed cross-location travel rows must be unique".into());
        }
    }
    Ok((
        RoutingProblem {
            couriers,
            shipments,
        },
        travel,
    ))
}

fn unique<'a>(ids: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = HashSet::new();
    for id in ids {
        if id.is_empty()
            || id.len() > 64
            || !id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || !seen.insert(id)
        {
            return Err("invalid or duplicate stable id".into());
        }
    }
    Ok(())
}
