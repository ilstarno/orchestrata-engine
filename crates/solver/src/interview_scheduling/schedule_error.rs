// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use core::fmt;

/// Input failures. They stop solving rather than quietly dropping requests.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScheduleError {
    InputSizeLimit,
    InvalidIdentifier,
    DuplicateIdentifier,
    InvalidFreeRange,
    InvalidRequest,
    TimeOverflow,
    InvalidPreviousPanel,
    LockedMeetingUnavailable,
    LockedMeetingsOverlap,
    PanelOptionLimit,
    InvalidSearchBudget,
}

impl ScheduleError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InputSizeLimit => "INT-001",
            Self::InvalidIdentifier => "INT-002",
            Self::DuplicateIdentifier => "INT-003",
            Self::InvalidFreeRange => "INT-004",
            Self::InvalidRequest => "INT-005",
            Self::TimeOverflow => "INT-006",
            Self::InvalidPreviousPanel => "INT-007",
            Self::LockedMeetingUnavailable => "INT-008",
            Self::LockedMeetingsOverlap => "INT-009",
            Self::PanelOptionLimit => "INT-010",
            Self::InvalidSearchBudget => "INT-011",
        }
    }

    pub const fn description(self) -> &'static str {
        match self {
            Self::InputSizeLimit => "Input exceeds the solver size limits.",
            Self::InvalidIdentifier => "An identifier is empty or not an opaque ASCII code.",
            Self::DuplicateIdentifier => {
                "A resource, request, role, or start is repeated where it must be unique."
            }
            Self::InvalidFreeRange => "Free ranges must be positive, sorted, and nonoverlapping.",
            Self::InvalidRequest => "An appointment has no required roles or has zero duration.",
            Self::TimeOverflow => "An appointment or buffer falls outside the UTC minute range.",
            Self::InvalidPreviousPanel => {
                "A prior panel does not contain one distinct person per role."
            }
            Self::LockedMeetingUnavailable => {
                "A locked meeting no longer satisfies the supplied plan constraints."
            }
            Self::LockedMeetingsOverlap => {
                "Locked meetings occupy a shared interviewer at overlapping times."
            }
            Self::PanelOptionLimit => {
                "Too many distinct panel choices; narrow the input or break up the batch."
            }
            Self::InvalidSearchBudget => {
                "Search budget must be between one and the maximum allowed nodes."
            }
        }
    }
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(out, "{}: {}", self.code(), self.description())
    }
}

impl core::error::Error for ScheduleError {}
