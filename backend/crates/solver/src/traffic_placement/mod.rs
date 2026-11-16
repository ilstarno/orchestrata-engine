// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

mod allocation_request;
mod apportion;
mod backend;
mod backend_allocation;
mod backend_disposition;
mod backend_share;
mod capacity_envelope;
mod dns_disposition;
mod dns_endpoint;
mod dns_policy;
mod dns_request;
mod dns_weight;
mod health;
mod health_snapshot;
mod plan_backend_allocation;
mod plan_dns_steering;
mod staleness_policy;
mod target_error;
mod unusable_reason;
mod validation;

pub use allocation_request::AllocationRequest;
pub use backend::Backend;
pub use backend_allocation::BackendAllocation;
pub use backend_disposition::BackendDisposition;
pub use backend_share::BackendShare;
pub use capacity_envelope::CapacityEnvelope;
pub use dns_disposition::DnsDisposition;
pub use dns_endpoint::DnsEndpoint;
pub use dns_policy::DnsPolicy;
pub use dns_request::DnsRequest;
pub use dns_weight::DnsWeight;
pub use health::Health;
pub use health_snapshot::HealthSnapshot;
pub use plan_backend_allocation::plan_backend_allocation;
pub use plan_dns_steering::{DNS_WEIGHT_SCALE, plan_dns_steering};
pub use staleness_policy::StalenessPolicy;
pub use target_error::TrafficError;
pub use unusable_reason::UnusableReason;
