use super::{
    CompiledRoute, DispatchError, DispatchPass, DispatchRoute, DispatchSolution, DispatchVisit,
    strategy::Strategy,
};
use orchestrata_solver::routing::{
    RouteConstruction, RouteConstructionStop, RoutingProblem, VisitKind, construct_routes,
};
fn stop(s: RouteConstructionStop) -> &'static str {
    match s {
        RouteConstructionStop::CompletedPass => "completed_pass",
        RouteConstructionStop::InsertionLimit => "insertion_limit",
    }
}
fn score(p: &RouteConstruction) -> (usize, u64) {
    (
        p.unassigned.len(),
        p.routes.iter().map(|r| r.depot_return - r.departure).sum(),
    )
}
pub fn solve_route_model(c: CompiledRoute) -> Result<DispatchSolution, DispatchError> {
    let travel = |mode, from, to| c.travel.get(&(mode, from, to)).copied();
    let mut selected =
        construct_routes(&c.problem, c.budget, travel).map_err(|_| DispatchError::NativeFailure)?;
    let mut evaluated = selected.evaluated;
    let mut passes = vec![DispatchPass {
        order: "input",
        evaluated_insertions: evaluated,
        stop: stop(selected.stop),
        selected: true,
    }];
    if c.spec.strategy == Strategy::TwoOrders && evaluated < c.budget {
        let mut order = (0..c.problem.shipments.len()).collect::<Vec<_>>();
        order.sort_by_key(|&i| {
            let s = &c.problem.shipments[i];
            (s.delivery.window.end, s.pickup.window.end, i)
        });
        if !order.iter().copied().eq(0..order.len()) {
            let alternative = RoutingProblem {
                couriers: c.problem.couriers.clone(),
                shipments: order
                    .iter()
                    .map(|&i| c.problem.shipments[i].clone())
                    .collect(),
            };
            // Any native error fails the request; never report an unknown trial count.
            let mut candidate = construct_routes(&alternative, c.budget - evaluated, travel)
                .map_err(|_| DispatchError::NativeFailure)?;
            evaluated += candidate.evaluated;
            let better = score(&candidate) < score(&selected);
            passes.push(DispatchPass {
                order: "delivery-window-end",
                evaluated_insertions: candidate.evaluated,
                stop: stop(candidate.stop),
                selected: better,
            });
            if better {
                for route in &mut candidate.routes {
                    for v in &mut route.visits {
                        v.shipment = order[v.shipment];
                    }
                }
                for i in &mut candidate.unassigned {
                    *i = order[*i];
                }
                candidate.unassigned.sort_unstable();
                selected = candidate;
                passes[0].selected = false;
            }
        }
    }
    let total_courier_minutes = score(&selected).1;
    let routes = selected
        .routes
        .iter()
        .map(|r| DispatchRoute {
            courier_id: c.snapshot.couriers[r.courier].id.clone(),
            depot_departure_minute: r.departure,
            depot_return_minute: r.depot_return,
            visits: r
                .visits
                .iter()
                .map(|v| {
                    let shipment = &c.snapshot.shipments[v.shipment];
                    let (kind, s) = match v.kind {
                        VisitKind::Pickup => ("pickup", &shipment.pickup),
                        VisitKind::Delivery => ("delivery", &shipment.delivery),
                    };
                    let start = v.arrival.max(s.window.start);
                    DispatchVisit {
                        shipment_id: shipment.id.clone(),
                        kind,
                        location: s.location.clone(),
                        arrival_minute: v.arrival,
                        service_start_minute: start,
                        service_end_minute: start + s.service_minutes,
                    }
                })
                .collect(),
        })
        .collect();
    Ok(DispatchSolution {
        schema: "route-dispatch-result-v1",
        algorithm: if c.spec.strategy == Strategy::InputOrder {
            "input-order-insertion"
        } else {
            "two-order-insertion"
        },
        guarantee: "verified-feasible-heuristic",
        time_unit: "minutes",
        load_unit: "kg",
        supplied_max_insertions: c.budget,
        evaluated_insertions: evaluated,
        total_courier_minutes,
        routes,
        unassigned: selected
            .unassigned
            .iter()
            .map(|&i| c.snapshot.shipments[i].id.clone())
            .collect(),
        passes,
    })
}
