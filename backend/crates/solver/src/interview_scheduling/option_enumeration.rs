// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{
    AppointmentRequest, Interviewer, ScheduleError, TimeRange, meeting_option::MeetingOption,
    validation::occupied_range,
};

const MAX_OPTIONS_PER_REQUEST: usize = 512;

fn is_available(person: &Interviewer<'_>, role: &str, interval: TimeRange) -> bool {
    person.roles.contains(&role) && person.free.iter().any(|range| range.contains(interval))
}

pub(super) fn locked_option(
    request: &AppointmentRequest<'_>,
    people: &[Interviewer<'_>],
) -> Result<MeetingOption, ScheduleError> {
    let unavailable = ScheduleError::LockedMeetingUnavailable;
    let prior = request.previous.ok_or(unavailable)?;
    if !request.allowed_starts.contains(&prior.start) {
        return Err(unavailable);
    }
    let occupied = occupied_range(request, prior.start)?;
    let index_panel = prior
        .panel
        .iter()
        .zip(request.roles)
        .map(|(id, role)| {
            people
                .iter()
                .position(|person| person.id == *id && is_available(person, role, occupied))
                .ok_or(unavailable)
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(MeetingOption {
        start: prior.start,
        index_panel,
        occupied,
        retained: true,
    })
}

fn visit_panels(
    choices: &[Vec<usize>],
    order: &[usize],
    selected: &mut [usize],
    occupied_people: u64,
    on_panel: &mut impl FnMut(&[usize]) -> Result<(), ScheduleError>,
) -> Result<(), ScheduleError> {
    let Some((&role, tail)) = order.split_first() else {
        return on_panel(selected);
    };
    for &index in &choices[role] {
        let bit = 1_u64 << index;
        if (occupied_people & bit) == 0 {
            selected[role] = index;
            visit_panels(choices, tail, selected, occupied_people | bit, on_panel)?;
        }
    }
    Ok(())
}

pub(super) fn available_options(
    request: &AppointmentRequest<'_>,
    people: &[Interviewer<'_>],
) -> Result<Vec<MeetingOption>, ScheduleError> {
    let mut options = Vec::new();
    for &start in request.allowed_starts {
        let occupied = occupied_range(request, start)?;
        let choices: Vec<Vec<usize>> = request
            .roles
            .iter()
            .map(|role| {
                people
                    .iter()
                    .enumerate()
                    .filter_map(|(index, person)| {
                        is_available(person, role, occupied).then_some(index)
                    })
                    .collect()
            })
            .collect();
        if choices.iter().any(Vec::is_empty) {
            continue;
        }
        let mut order: Vec<_> = (0..choices.len()).collect();
        order.sort_unstable_by_key(|&index| (choices[index].len(), index));
        let mut selected = vec![0; choices.len()];
        visit_panels(&choices, &order, &mut selected, 0, &mut |indices| {
            if options.len() == MAX_OPTIONS_PER_REQUEST {
                return Err(ScheduleError::PanelOptionLimit);
            }
            let retained = request.previous.is_some_and(|prior| {
                prior.start == start
                    && prior
                        .panel
                        .iter()
                        .copied()
                        .eq(indices.iter().map(|&index| people[index].id))
            });
            options.push(MeetingOption {
                start,
                index_panel: indices.to_vec(),
                occupied,
                retained,
            });
            Ok(())
        })?;
    }
    options.sort_unstable_by(|a, b| {
        b.retained
            .cmp(&a.retained)
            .then(a.start.cmp(&b.start))
            .then(a.index_panel.cmp(&b.index_panel))
    });
    Ok(options)
}
