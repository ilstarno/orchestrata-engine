// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use super::{
    all_different_scratch::AllDifferentScratch, no_overlap_scratch::NoOverlapScratch,
    resource_load_totals::ResourceLoadTotals,
};

/// Temporary storage reused by a single exact search.
#[derive(Default)]
pub(super) struct CandidateScratch {
    pub(super) all_different: AllDifferentScratch,
    pub(super) no_overlap: NoOverlapScratch,
    pub(super) resource_loads: ResourceLoadTotals,
}
