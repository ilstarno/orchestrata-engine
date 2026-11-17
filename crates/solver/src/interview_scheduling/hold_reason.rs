// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Explain why a meeting has no proposed assignment, not why an applicant
/// should advance or be rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldReason {
    /// No start was offered by the scheduling caller.
    NoOfferedStart,
    /// No offered start can be staffed by distinct, qualified, free panelists.
    NoQualifiedPanel,
    /// There is a valid panel in isolation but nothing fits the proven plan.
    PanelConflictsWithPlan,
    /// Search hit its budget; a valid schedule may still exist.
    SearchBudgetReached,
}
