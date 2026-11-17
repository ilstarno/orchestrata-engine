// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{TimeRange, meeting_option::MeetingOption};

pub(super) struct ScheduleBookings {
    busy: Vec<Vec<TimeRange>>,
}

impl ScheduleBookings {
    pub(super) fn new(interviewers: usize) -> Self {
        Self {
            busy: vec![Vec::new(); interviewers],
        }
    }

    pub(super) fn fits(&self, proposed: &MeetingOption) -> bool {
        proposed.index_panel.iter().all(|&index| {
            self.busy[index]
                .iter()
                .all(|meeting| !meeting.overlaps(proposed.occupied))
        })
    }

    pub(super) fn add(&mut self, proposed: &MeetingOption) {
        for &index in &proposed.index_panel {
            self.busy[index].push(proposed.occupied);
        }
    }

    pub(super) fn remove_last(&mut self, proposed: &MeetingOption) {
        for &index in &proposed.index_panel {
            self.busy[index].pop();
        }
    }
}
