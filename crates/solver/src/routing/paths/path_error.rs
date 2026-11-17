// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

/// Invalid graph/search inputs and distinct failures that are not "unreachable".
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PathError {
    UnknownNode(usize),
    NegativeTravelTime { road_index: usize },
    WrongEstimateCount { expected: usize, actual: usize },
    NonzeroGoalEstimate,
    InconsistentEstimate { from: usize, to: usize },
    ExpansionLimitReached { expanded: usize },
    TravelTimeOverflow,
}
