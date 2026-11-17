// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Status supplied by the operator; the solver itself does not monitor hosts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Health {
    Healthy,
    Unhealthy,
    Unknown,
}
