mod compile;
mod compiled_route;
mod courier_row;
mod dispatch_error;
mod dispatch_pass;
mod dispatch_route;
mod dispatch_solution;
mod dispatch_visit;
mod leg_row;
mod mode;
mod parse;
mod route_spec;
mod shipment_row;
mod snapshot;
mod solve;
mod stop_row;
mod strategy;
mod window;
pub use compiled_route::CompiledRoute;
pub use dispatch_error::DispatchError;
pub use dispatch_pass::DispatchPass;
pub use dispatch_route::DispatchRoute;
pub use dispatch_solution::DispatchSolution;
pub use dispatch_visit::DispatchVisit;
pub use parse::{compile_route_model, parse_route_model};
pub use route_spec::RouteSpec;
pub use solve::solve_route_model;

mod coverage;
mod coverage_error;
pub use coverage::certify_route_coverage;
pub use coverage_error::DispatchCoverageError;
pub use orchestrata_solver::routing::coverage::{
    CoverageCertificate, CoverageOutcome, MAX_COVERAGE_WORK,
};
