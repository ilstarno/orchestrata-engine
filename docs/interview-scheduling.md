# Schedule interviews

The Rust scheduling routine takes interviews already approved by a recruiter and suggests times and interviewer panels. It handles conflicts and explains holds. It does not read applications or personal records, score people, decide who advances, or create calendar invitations.

Inputs use opaque request and interviewer codes. Supply offered starts as UTC minutes and normalize each interviewer’s free calendar windows to sorted, nonoverlapping, half-open intervals. Keep candidate availability, work-hour rules, and any room/meeting-link reservations in the calling scheduling workflow. A role appears once for each interviewer needed; repeating a role requires distinct people. Buffers occupy interviewers before and after the meeting. Existing confirmed meetings can be marked locked. An invalid lock stops the solve rather than silently moving it.

The objective is to schedule as many interviews as possible, then retain existing appointment suggestions where possible. Changes include moved times, changed panels and holds of previously proposed meetings. The result can explain a missing offered start, unavailable qualified panel, scheduling conflict, or search budget exhaustion. A result with `proven_optimal = false` is a feasible suggestion, not evidence that another appointment cannot fit.

```rust
use orchestrata_solver::interview_scheduling::{
    AppointmentRequest, Interviewer, TimeRange, schedule_interviews,
};

let free = [TimeRange { start: 600, end: 660 }];
let panel = [
    Interviewer { id: "staff-01", roles: &["host"], free: &free },
    Interviewer { id: "staff-02", roles: &["reviewer"], free: &free },
];
let interviews = [AppointmentRequest {
    id: "interview-01",
    roles: &["host", "reviewer"],
    allowed_starts: &[605, 615],
    duration: 30,
    before: 5,
    after: 5,
    previous: None,
    locked: false,
}];
let result = schedule_interviews(&interviews, &panel, 10_000)?;
assert!(result.proven_optimal);
# Ok::<(), orchestrata_solver::interview_scheduling::ScheduleError>(())
```

This release is a bounded Rust routine for one scheduling batch. It accepts up to 32 interviews, 64 interviewer resources, four roles and 32 proposed starts per interview, 32 roles and 64 free windows per interviewer, and 512 feasible panel choices per interview. A narrower batch or fewer offered combinations may be necessary. The search budget must be between 1 and 200,000 states. No REST, FFI, or SDK binding is included here.

| Code | Meaning |
| --- | --- |
| INT-001 | Input exceeds the solver size limits. |
| INT-002 | An identifier is empty or not an opaque ASCII code. |
| INT-003 | A resource, request, role, or start is repeated where it must be unique. |
| INT-004 | Free ranges must be positive, sorted, and nonoverlapping. |
| INT-005 | An appointment has no required roles or has zero duration. |
| INT-006 | An appointment or buffer falls outside the UTC minute range. |
| INT-007 | A prior panel does not contain one distinct person per role. |
| INT-008 | A locked meeting no longer satisfies the supplied plan constraints. |
| INT-009 | Locked meetings occupy a shared interviewer at overlapping times. |
| INT-010 | Too many distinct panel choices; narrow the input or break up the batch. |
| INT-011 | Search budget must be between one and the maximum allowed nodes. |
