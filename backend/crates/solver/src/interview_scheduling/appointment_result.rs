// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{Appointment, HoldReason};

/// An appointment suggestion or an explainable hold for the coordinator.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppointmentResult<'a> {
    Proposed {
        id: &'a str,
        appointment: Appointment<'a>,
    },
    Held {
        id: &'a str,
        reason: HoldReason,
    },
}
