// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{meeting_option::MeetingOption, schedule_bookings::ScheduleBookings};

/// Incumbent score is lexicographic: maximize scheduled appointments, then
/// minimize disruption to previous proposals. Equal scores keep the first
/// schedule encountered in the stable exploration order.
pub(super) struct Search<'a> {
    options: &'a [Vec<MeetingOption>],
    prior: &'a [bool],
    order: &'a [usize],
    budget: u32,
    pub(super) visited: u32,
    pub(super) exhausted: bool,
    pub(super) best: Vec<Option<usize>>,
    pub(super) changed: usize,
    scheduled: usize,
}

impl<'a> Search<'a> {
    pub(super) fn new(
        options: &'a [Vec<MeetingOption>],
        prior: &'a [bool],
        order: &'a [usize],
        budget: u32,
        initial: Vec<Option<usize>>,
    ) -> Self {
        Self {
            options,
            prior,
            order,
            budget,
            visited: 0,
            exhausted: false,
            scheduled: initial.iter().flatten().count(),
            changed: order.iter().filter(|&&id| prior[id]).count(),
            best: initial,
        }
    }

    pub(super) fn run(&mut self, bookings: &mut ScheduleBookings) {
        let mut current = self.best.clone();
        self.visit(&mut current, bookings, 0, self.scheduled, 0);
    }

    fn visit(
        &mut self,
        current: &mut [Option<usize>],
        bookings: &mut ScheduleBookings,
        depth: usize,
        scheduled: usize,
        changed: usize,
    ) {
        let potential = scheduled + self.order.len() - depth;
        if potential < self.scheduled || (potential == self.scheduled && changed >= self.changed) {
            return;
        }
        if self.visited == self.budget {
            self.exhausted = true;
            return;
        }
        self.visited += 1;
        let Some(&request_index) = self.order.get(depth) else {
            self.scheduled = scheduled;
            self.changed = changed;
            self.best.clone_from_slice(current);
            return;
        };
        for (index, option) in self.options[request_index].iter().enumerate() {
            if bookings.fits(option) {
                bookings.add(option);
                current[request_index] = Some(index);
                let disruption = usize::from(self.prior[request_index] && !option.retained);
                self.visit(
                    current,
                    bookings,
                    depth + 1,
                    scheduled + 1,
                    changed + disruption,
                );
                current[request_index] = None;
                bookings.remove_last(option);
                if self.exhausted {
                    return;
                }
            }
        }
        self.visit(
            current,
            bookings,
            depth + 1,
            scheduled,
            changed + usize::from(self.prior[request_index]),
        );
    }
}
