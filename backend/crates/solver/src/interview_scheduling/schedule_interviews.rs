// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{
    Appointment, AppointmentRequest, AppointmentResult, HoldReason, InterviewSchedule, Interviewer,
    ScheduleError,
    option_enumeration::{available_options, locked_option},
    schedule_bookings::ScheduleBookings,
    search::Search,
    validation::validate,
};

/// Maximum accepted budget for branch-and-bound states.
pub const MAX_INTERVIEW_SEARCH_NODES: u32 = 200_000;

/// Schedule already-approved interviews without overlapping interviewers.
///
/// All times are normalized UTC minutes and all identities are opaque codes.
/// Locked meetings are preserved exactly or the request is rejected. The
/// search maximizes scheduled meetings then retains existing suggestions where
/// possible. A limited search returns its feasible incumbent, not a guarantee
/// of optimality. The result is a suggestion; it never writes to a calendar.
pub fn schedule_interviews<'a>(
    requests: &[AppointmentRequest<'a>],
    people: &[Interviewer<'a>],
    search_budget: u32,
) -> Result<InterviewSchedule<'a>, ScheduleError> {
    if !(1..=MAX_INTERVIEW_SEARCH_NODES).contains(&search_budget) {
        return Err(ScheduleError::InvalidSearchBudget);
    }
    validate(requests, people)?;
    let mut people = people.to_vec();
    people.sort_unstable_by_key(|person| person.id);
    let mut requests = requests.to_vec();
    requests.sort_unstable_by_key(|request| request.id);

    let mut booked = ScheduleBookings::new(people.len());
    let mut start = vec![None; requests.len()];
    let mut options = Vec::with_capacity(requests.len());
    let mut order = Vec::new();
    for (index, request) in requests.iter().enumerate() {
        if request.locked {
            let selected = locked_option(request, &people)?;
            if !booked.fits(&selected) {
                return Err(ScheduleError::LockedMeetingsOverlap);
            }
            booked.add(&selected);
            start[index] = Some(0);
            options.push(vec![selected]);
        } else {
            order.push(index);
            options.push(available_options(request, &people)?);
        }
    }
    order.sort_unstable_by_key(|&index| (options[index].len(), requests[index].id));
    let prior: Vec<_> = requests
        .iter()
        .map(|request| request.previous.is_some())
        .collect();

    let mut search = Search::new(&options, &prior, &order, search_budget, start);
    search.run(&mut booked);
    let appointments = requests
        .iter()
        .enumerate()
        .map(|(index, request)| {
            let id = request.id;
            match search.best[index] {
                Some(selected) => {
                    let choice = &options[index][selected];
                    AppointmentResult::Proposed {
                        id,
                        appointment: Appointment {
                            start: choice.start,
                            panel: choice
                                .index_panel
                                .iter()
                                .map(|&person| people[person].id)
                                .collect(),
                        },
                    }
                }
                None => AppointmentResult::Held {
                    id,
                    reason: if request.allowed_starts.is_empty() {
                        HoldReason::NoOfferedStart
                    } else if options[index].is_empty() {
                        HoldReason::NoQualifiedPanel
                    } else if search.exhausted {
                        HoldReason::SearchBudgetReached
                    } else {
                        HoldReason::PanelConflictsWithPlan
                    },
                },
            }
        })
        .collect();
    Ok(InterviewSchedule {
        appointments,
        changed_previous: search.changed,
        proven_optimal: !search.exhausted,
        search_nodes: search.visited,
    })
}
