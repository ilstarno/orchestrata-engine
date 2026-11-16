// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnusableReason {
    NotAdmitted,
    ReportedUnhealthy,
    UnknownHealth,
    MissingHealthCheck,
    StaleHealthCheck,
    NoHeadroom,
}
