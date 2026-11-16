// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use orchestrata_solver::interview_scheduling::{
    Appointment, AppointmentRequest, AppointmentResult, HoldReason, Interviewer, ScheduleError,
    TimeRange, schedule_interviews,
};

const ANY_TIME: [TimeRange; 1] = [TimeRange {
    start: 450,
    end: 850,
}];

fn appointment<'a>(id: &'a str, roles: &'a [&'a str], starts: &'a [u64]) -> AppointmentRequest<'a> {
    AppointmentRequest {
        id,
        roles,
        allowed_starts: starts,
        duration: 30,
        before: 0,
        after: 0,
        previous: None,
        locked: false,
    }
}

fn person<'a>(id: &'a str, roles: &'a [&'a str]) -> Interviewer<'a> {
    Interviewer {
        id,
        roles,
        free: &ANY_TIME,
    }
}

fn proposal<'a>(id: &'a str, start: u64, panel: &[&'a str]) -> AppointmentResult<'a> {
    AppointmentResult::Proposed {
        id,
        appointment: Appointment {
            start,
            panel: panel.to_vec(),
        },
    }
}

#[test]
fn finds_maximum_coverage_and_is_independent_of_input_order() {
    // A first-fit assignment of A to host-1 leaves B with no interviewer.
    let a = appointment("A", &["technical"], &[500, 600]);
    let b = appointment("B", &["lead"], &[500]);
    let first = person("host-1", &["technical", "lead"]);
    let second = person("host-2", &["technical"]);
    let schedule = schedule_interviews(&[b, a], &[second, first], 10_000).unwrap();
    assert!(schedule.proven_optimal);
    assert_eq!(
        schedule.appointments,
        [
            proposal("A", 500, &["host-2"]),
            proposal("B", 500, &["host-1"]),
        ]
    );
    assert_eq!(
        schedule,
        schedule_interviews(&[a, b], &[first, second], 10_000).unwrap()
    );
}

#[test]
fn reschedule_minimizes_churn_after_coverage_and_preserves_locks() {
    let still_locked = Appointment {
        start: 500,
        panel: vec!["host-1"],
    };
    let still_feasible = Appointment {
        start: 645,
        panel: vec!["host-2"],
    };
    let reschedule = Appointment {
        start: 600,
        panel: vec!["host-offline"],
    };
    let requests = [
        AppointmentRequest {
            previous: Some(&still_locked),
            locked: true,
            ..appointment("A", &["lead"], &[500])
        },
        AppointmentRequest {
            previous: Some(&reschedule),
            ..appointment("B", &["lead"], &[600, 645])
        },
        AppointmentRequest {
            previous: Some(&still_feasible),
            ..appointment("C", &["lead"], &[600, 645])
        },
    ];
    let schedule = schedule_interviews(
        &requests,
        &[person("host-1", &["lead"]), person("host-2", &["lead"])],
        10_000,
    )
    .unwrap();
    assert!(schedule.proven_optimal);
    assert_eq!(schedule.changed_previous, 1);
    assert!(
        schedule
            .appointments
            .contains(&proposal("A", 500, &["host-1"]))
    );
    assert!(
        schedule
            .appointments
            .contains(&proposal("C", 645, &["host-2"]))
    );
}

#[test]
fn no_offered_start_and_different_panel_failure_modes_are_audited() {
    let requests = [
        appointment("A", &["lead"], &[]),
        appointment("B", &["lead", "lead"], &[500]),
        appointment("C", &["lead"], &[500]),
        appointment("D", &["lead"], &[500]),
    ];
    let people = [person("host", &["lead"])];
    let schedule = schedule_interviews(&requests, &people, 10_000).unwrap();
    assert!(schedule.proven_optimal);
    assert_eq!(
        schedule.appointments,
        [
            AppointmentResult::Held {
                id: "A",
                reason: HoldReason::NoOfferedStart
            },
            AppointmentResult::Held {
                id: "B",
                reason: HoldReason::NoQualifiedPanel
            },
            proposal("C", 500, &["host"]),
            AppointmentResult::Held {
                id: "D",
                reason: HoldReason::PanelConflictsWithPlan
            },
        ],
    );
    let limited = schedule_interviews(&requests, &people, 1).unwrap();
    assert_eq!(limited.search_nodes, 1);
    assert!(!limited.proven_optimal);
    assert!(limited.appointments.contains(&AppointmentResult::Held {
        id: "D",
        reason: HoldReason::SearchBudgetReached
    }));
}

#[test]
fn distinct_role_slots_and_half_open_buffer_occupancy_are_enforced() {
    let first = AppointmentRequest {
        after: 10,
        ..appointment("A", &["lead", "lead"], &[500])
    };
    let second = AppointmentRequest {
        before: 5,
        ..appointment("B", &["lead"], &[530, 545])
    };
    let close = [TimeRange {
        start: 500,
        end: 540,
    }];
    let p1 = person("host-1", &["lead"]);
    let p2 = Interviewer {
        free: &close,
        ..person("host-2", &["lead"])
    };
    let schedule = schedule_interviews(&[first, second], &[p1, p2], 40).unwrap();
    assert_eq!(
        schedule.appointments,
        [
            proposal("A", 500, &["host-1", "host-2"]),
            proposal("B", 545, &["host-1"])
        ],
    );
}

#[test]
fn invalid_locked_meetings_do_not_silently_change() {
    let prior = Appointment {
        start: 500,
        panel: vec!["host"],
    };
    let a = AppointmentRequest {
        previous: Some(&prior),
        locked: true,
        ..appointment("A", &["lead"], &[500])
    };
    let b = AppointmentRequest { id: "B", ..a };
    let people = [person("host", &["lead"])];
    let conflict = schedule_interviews(&[a, b], &people, 40).unwrap_err();
    assert_eq!(conflict, ScheduleError::LockedMeetingsOverlap);
    assert_eq!(conflict.code(), "INT-009");
    assert_eq!(
        schedule_interviews(&[a], &[], 40).unwrap_err(),
        ScheduleError::LockedMeetingUnavailable,
    );
}

#[test]
fn rejects_bad_calendars_overflows_and_unbounded_enumeration() {
    let invalid = [
        TimeRange {
            start: 460,
            end: 650,
        },
        TimeRange {
            start: 590,
            end: 700,
        },
    ];
    let person = Interviewer {
        id: "host-1",
        roles: &["lead"],
        free: &invalid,
    };
    let a = appointment("A", &["lead"], &[500]);
    assert_eq!(
        schedule_interviews(&[a], &[person], 40).unwrap_err().code(),
        "INT-004"
    );
    assert_eq!(
        schedule_interviews(&[a], &[], 0).unwrap_err().code(),
        "INT-011"
    );
    let overflow = appointment("A", &["lead"], &[u64::MAX]);
    assert_eq!(
        schedule_interviews(&[overflow], &[], 40)
            .unwrap_err()
            .code(),
        "INT-006"
    );
    let ids: Vec<_> = (0..16).map(|i| format!("host-{i}")).collect();
    let people: Vec<_> = ids.iter().map(|id| self::person(id, &["lead"])).collect();
    let wide = appointment("A", &["lead", "lead", "lead", "lead"], &[500]);
    assert_eq!(
        schedule_interviews(&[wide], &people, 40)
            .unwrap_err()
            .code(),
        "INT-010"
    );
}

#[test]
fn scores_match_exhaustive_appointment_oracle() {
    // Independently enumerate three request placements across two hosts and two
    // disjoint starts. Busy cells are a bitset, not the engine's interval code.
    let first_only = [TimeRange {
        start: 500,
        end: 530,
    }];
    let later_only = [TimeRange {
        start: 530,
        end: 560,
    }];
    let prior = Appointment {
        start: 500,
        panel: vec!["host-2"],
    };
    let menus = [vec![], vec![500], vec![530], vec![500, 530]];
    for mask in 0..64 {
        let providers = [
            Interviewer {
                free: &first_only,
                ..person("host-1", &["lead"])
            },
            Interviewer {
                free: if mask % 2 == 0 {
                    &later_only
                } else {
                    &ANY_TIME
                },
                ..person("host-2", &["lead"])
            },
        ];
        let chosen = [
            appointment("A", &["lead"], &menus[mask % 4]),
            appointment("B", &["lead"], &menus[mask / 4 % 4]),
            AppointmentRequest {
                previous: Some(&prior),
                ..appointment("C", &["lead"], &menus[mask / 16])
            },
        ];
        let solution = schedule_interviews(&chosen, &providers, 50_000).unwrap();
        assert!(solution.proven_optimal);
        // Four cells: host-1 first/later; host-2 first/later. 4 is a hold.
        let legal = |index: usize, cell: usize| match cell {
            0 => chosen[index].allowed_starts.contains(&500),
            1 => false,
            2 => mask % 2 != 0 && chosen[index].allowed_starts.contains(&500),
            3 => chosen[index].allowed_starts.contains(&530),
            _ => true,
        };
        let mut best = (0, 0);
        for a in 0..5 {
            for b in 0..5 {
                for c in 0..5 {
                    let cells = [a, b, c];
                    if !cells
                        .into_iter()
                        .enumerate()
                        .all(|(index, cell)| legal(index, cell))
                    {
                        continue;
                    }
                    let booked: Vec<_> = cells.into_iter().filter(|&cell| cell < 4).collect();
                    if booked
                        .iter()
                        .any(|&cell| booked.iter().filter(|&&x| x == cell).count() > 1)
                    {
                        continue;
                    }
                    best = best.max((booked.len(), usize::from(c == 2)));
                }
            }
        }
        let proposed = solution
            .appointments
            .iter()
            .filter(|row| matches!(row, AppointmentResult::Proposed { .. }))
            .count();
        assert_eq!(
            (proposed, 1 - solution.changed_previous),
            best,
            "matrix {mask}"
        );
    }
}
