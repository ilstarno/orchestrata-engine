// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// One meeting proposal. Panel order matches the requested role positions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Appointment<'a> {
    pub start: u64,
    pub panel: Vec<&'a str>,
}
