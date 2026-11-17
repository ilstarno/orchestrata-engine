// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::Appointment;

/// One interview already approved for scheduling.
/// Legal start times are supplied by the caller; repeated role codes require
/// distinct panelists. Buffers occupy panelists but do not extend the meeting.
#[derive(Clone, Copy, Debug)]
pub struct AppointmentRequest<'a> {
    pub id: &'a str,
    pub roles: &'a [&'a str],
    pub allowed_starts: &'a [u64],
    pub duration: u32,
    pub before: u32,
    pub after: u32,
    pub previous: Option<&'a Appointment<'a>>,
    pub locked: bool,
}
