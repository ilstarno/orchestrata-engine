// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use std::collections::HashSet;

use super::{
    OwnerRequirements, OwnerSelection, OwnerSelectionError, OwnerSelectionReason, Representative,
};

/// Selects an available exact territory and language match, then minimizes
/// workload and breaks equal-workload ties by ascending opaque ASCII ID.
/// Matching never trims or folds codes. Candidate IDs must be unique and use
/// only ASCII letters, digits, or `-`, `_`, `.`, and `:`.
pub fn select_owner<'a>(
    requirements: OwnerRequirements<'_>,
    representatives: &[Representative<'a>],
) -> Result<OwnerSelection<'a>, OwnerSelectionError> {
    if requirements.territory.trim().is_empty() || requirements.language.trim().is_empty() {
        return Err(OwnerSelectionError::BlankRequirements);
    }

    let mut ids = HashSet::with_capacity(representatives.len());
    let mut best: Option<Representative<'a>> = None;
    let mut eligible_count = 0;
    let mut minimum_count = 0;
    for &candidate in representatives {
        if !valid_id(candidate.id) {
            return Err(OwnerSelectionError::InvalidRepresentativeId);
        }
        if !ids.insert(candidate.id) {
            return Err(OwnerSelectionError::DuplicateRepresentativeId);
        }
        if !candidate.available
            || candidate.territory != requirements.territory
            || candidate.language != requirements.language
        {
            continue;
        }

        eligible_count += 1;
        match best {
            Some(current) if candidate.workload > current.workload => {}
            Some(current) if candidate.workload == current.workload => {
                minimum_count += 1;
                if candidate.id < current.id {
                    best = Some(candidate);
                }
            }
            _ => {
                minimum_count = 1;
                best = Some(candidate);
            }
        }
    }

    let Some(selected) = best else {
        return Ok(OwnerSelection::NoEligibleRepresentative);
    };
    let reason = if eligible_count == 1 {
        OwnerSelectionReason::OnlyEligible
    } else if minimum_count == 1 {
        OwnerSelectionReason::LeastWorkload
    } else {
        OwnerSelectionReason::StableIdTieBreak
    };

    Ok(OwnerSelection::Assigned {
        representative_id: selected.id,
        workload: selected.workload,
        eligible_count,
        reason,
    })
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
}
