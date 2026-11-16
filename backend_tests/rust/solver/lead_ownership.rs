// SPDX-License-Identifier: Apache-2.0
// @author Orchestrata contributors

use orchestrata_solver::lead_ownership::{
    OwnerRequirements, OwnerSelection, OwnerSelectionError, OwnerSelectionReason, Representative,
    select_owner,
};

const REQUIREMENTS: OwnerRequirements<'static> = OwnerRequirements {
    territory: "DE",
    language: "de",
};

fn rep(id: &str, workload: u64) -> Representative<'_> {
    Representative {
        id,
        territory: "DE",
        language: "de",
        available: true,
        workload,
    }
}

#[test]
fn eligibility_requires_every_exact_match_before_considering_workload() {
    let candidates = [
        Representative {
            available: false,
            ..rep("rep-01", 0)
        },
        Representative {
            territory: "NL",
            ..rep("rep-02", 0)
        },
        Representative {
            language: "en",
            ..rep("rep-03", 0)
        },
        Representative {
            territory: "de",
            ..rep("rep-04", 0)
        },
        Representative {
            language: "DE",
            ..rep("rep-05", 0)
        },
        rep("rep-06", u64::MAX),
    ];

    assert_eq!(
        select_owner(REQUIREMENTS, &candidates),
        Ok(OwnerSelection::Assigned {
            representative_id: "rep-06",
            workload: u64::MAX,
            eligible_count: 1,
            reason: OwnerSelectionReason::OnlyEligible,
        }),
    );
}

#[test]
fn chooses_least_workload_before_id_and_ignores_ties_at_worse_workloads() {
    let candidates = [rep("rep-01", 4), rep("rep-03", 4), rep("rep-02", 1)];
    assert_eq!(
        select_owner(REQUIREMENTS, &candidates),
        Ok(OwnerSelection::Assigned {
            representative_id: "rep-02",
            workload: 1,
            eligible_count: 3,
            reason: OwnerSelectionReason::LeastWorkload,
        }),
    );
}

#[test]
fn tie_break_and_explanation_do_not_depend_on_candidate_order() {
    let candidates = [rep("rep-02", 3), rep("rep-01", 3), rep("rep-00", 4)];
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        let arranged = order.map(|index| candidates[index]);
        assert_eq!(
            select_owner(REQUIREMENTS, &arranged),
            Ok(OwnerSelection::Assigned {
                representative_id: "rep-01",
                workload: 3,
                eligible_count: 3,
                reason: OwnerSelectionReason::StableIdTieBreak,
            }),
        );
    }
}

#[test]
fn no_match_is_a_decision_not_an_input_error() {
    assert_eq!(
        select_owner(REQUIREMENTS, &[]),
        Ok(OwnerSelection::NoEligibleRepresentative)
    );
    let candidates = [Representative {
        language: "fr",
        ..rep("rep-01", 0)
    }];
    assert_eq!(
        select_owner(REQUIREMENTS, &candidates),
        Ok(OwnerSelection::NoEligibleRepresentative)
    );
}

#[test]
fn rejects_blank_requirements_and_ambiguous_or_invalid_ids() {
    let blank = OwnerRequirements {
        territory: " ",
        ..REQUIREMENTS
    };
    assert_eq!(
        select_owner(blank, &[]),
        Err(OwnerSelectionError::BlankRequirements)
    );
    let duplicated = [
        rep("rep-01", 1),
        Representative {
            available: false,
            ..rep("rep-01", 3)
        },
    ];
    assert_eq!(
        select_owner(REQUIREMENTS, &duplicated),
        Err(OwnerSelectionError::DuplicateRepresentativeId)
    );
    for invalid in ["", "non opaque", "non@opaque", "R–1"] {
        assert_eq!(
            select_owner(REQUIREMENTS, &[rep(invalid, 0)]),
            Err(OwnerSelectionError::InvalidRepresentativeId),
        );
    }
}
