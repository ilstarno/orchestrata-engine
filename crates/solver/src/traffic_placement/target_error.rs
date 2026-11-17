// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use core::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrafficError {
    InputSizeLimit,
    InvalidIdentifier,
    DuplicateEndpoint,
    DuplicateRegion,
    FutureHealthCheck,
    EmptyApprovedRegions,
}

impl TrafficError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputSizeLimit => "TRF-001",
            Self::InvalidIdentifier => "TRF-002",
            Self::DuplicateEndpoint => "TRF-003",
            Self::DuplicateRegion => "TRF-004",
            Self::FutureHealthCheck => "TRF-005",
            Self::EmptyApprovedRegions => "TRF-006",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::InputSizeLimit => "The placement input exceeds the allowed dimensions.",
            Self::InvalidIdentifier => {
                "An endpoint, region, or pool key is not a valid opaque identifier."
            }
            Self::DuplicateEndpoint => {
                "Target identifiers must be unique within the supplied snapshot."
            }
            Self::DuplicateRegion => "An approved steering region appears more than once.",
            Self::FutureHealthCheck => "A health check occurs after the snapshot evaluation time.",
            Self::EmptyApprovedRegions => {
                "A steering policy must have at least one approved region."
            }
        }
    }
}

impl fmt::Display for TrafficError {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(output, "{}: {}", self.code(), self.description())
    }
}

impl core::error::Error for TrafficError {}
