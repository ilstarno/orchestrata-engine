// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::TimeRange;

/// Calendar resource with role codes and already-normalized free windows.
/// Deliberately does not contain any applicant attributes.
#[derive(Clone, Copy, Debug)]
pub struct Interviewer<'a> {
    pub id: &'a str,
    pub roles: &'a [&'a str],
    pub free: &'a [TimeRange],
}
