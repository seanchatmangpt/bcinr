//! Falsifiers for machine-experience compile-back (A2A-2612).
//!
//! Before v26.9.26 `experience.rs` existed in the tree but was not declared as a
//! module, so none of this contract was compiled or exercised. These tests keep it
//! reachable. Chicago style: real functions, state assertions, no doubles.

use bcinr_mfw_ir::{
    candidate_from_execution, qualify_candidate, Digest, ExperienceAuthority, ExperienceEvidence,
    ExperienceRefusal,
};

fn d(value: &str) -> Digest {
    Digest::hash(value.as_bytes())
}

fn evidence() -> ExperienceEvidence {
    ExperienceEvidence {
        semantic_subject: d("subject"),
        execution_receipt: d("exec-receipt"),
        execution_succeeded: true,
        ontology_digest: d("ontology"),
        manufacturer_digest: d("manufacturer"),
    }
}

#[test]
fn failed_execution_never_becomes_a_candidate() {
    let mut ev = evidence();
    ev.execution_succeeded = false;
    assert_eq!(
        candidate_from_execution(&ev),
        Err(ExperienceRefusal::ExecutionNotSuccessful)
    );
}

#[test]
fn candidate_carries_no_authority_and_is_deterministic() {
    let a = candidate_from_execution(&evidence()).unwrap();
    let b = candidate_from_execution(&evidence()).unwrap();
    assert_eq!(a, b);
    assert_eq!(a.authority(), ExperienceAuthority::None);
    assert_eq!(a.source_execution_receipt(), d("exec-receipt"));
}

#[test]
fn every_semantic_input_changes_candidate_identity() {
    let base = candidate_from_execution(&evidence())
        .unwrap()
        .candidate_id();
    let mutations: [fn(&mut ExperienceEvidence); 4] = [
        |e| e.semantic_subject = d("subject-2"),
        |e| e.execution_receipt = d("exec-receipt-2"),
        |e| e.ontology_digest = d("ontology-2"),
        |e| e.manufacturer_digest = d("manufacturer-2"),
    ];
    for mutate in mutations {
        let mut ev = evidence();
        mutate(&mut ev);
        assert_ne!(candidate_from_execution(&ev).unwrap().candidate_id(), base);
    }
}

#[test]
fn qualification_requires_both_receipts() {
    let c = candidate_from_execution(&evidence()).unwrap();
    assert_eq!(
        qualify_candidate(
            &c,
            d("ontology"),
            d("manufacturer"),
            None,
            Some(d("verify"))
        ),
        Err(ExperienceRefusal::MissingAdmissionReceipt)
    );
    assert_eq!(
        qualify_candidate(&c, d("ontology"), d("manufacturer"), Some(d("admit")), None),
        Err(ExperienceRefusal::MissingVerificationReceipt)
    );
}

#[test]
fn stale_ontology_or_manufacturer_identity_is_refused() {
    let c = candidate_from_execution(&evidence()).unwrap();
    assert_eq!(
        qualify_candidate(
            &c,
            d("ontology-2"),
            d("manufacturer"),
            Some(d("admit")),
            Some(d("verify"))
        ),
        Err(ExperienceRefusal::OntologyIdentityChanged)
    );
    assert_eq!(
        qualify_candidate(
            &c,
            d("ontology"),
            d("manufacturer-2"),
            Some(d("admit")),
            Some(d("verify"))
        ),
        Err(ExperienceRefusal::ManufacturerIdentityChanged)
    );
}

/// Execution success is not standing: the execution receipt cannot double as the
/// admission or verification receipt, and one receipt cannot serve as both.
#[test]
fn non_independent_receipts_are_refused() {
    let c = candidate_from_execution(&evidence()).unwrap();
    let exec = d("exec-receipt");
    for (admit, verify) in [
        (d("same"), d("same")),
        (exec, d("verify")),
        (d("admit"), exec),
    ] {
        assert_eq!(
            qualify_candidate(
                &c,
                d("ontology"),
                d("manufacturer"),
                Some(admit),
                Some(verify)
            ),
            Err(ExperienceRefusal::NonIndependentReceipts)
        );
    }
}

#[test]
fn qualified_capability_is_bound_to_receipts_and_has_no_authority() {
    let c = candidate_from_execution(&evidence()).unwrap();
    let q = qualify_candidate(
        &c,
        d("ontology"),
        d("manufacturer"),
        Some(d("admit")),
        Some(d("verify")),
    )
    .unwrap();
    assert_eq!(q.authority(), ExperienceAuthority::None);
    assert_eq!(q.promotion_candidate_id(), c.candidate_id());
    assert_ne!(q.capability_id(), c.candidate_id());
    let swapped = qualify_candidate(
        &c,
        d("ontology"),
        d("manufacturer"),
        Some(d("verify")),
        Some(d("admit")),
    )
    .unwrap();
    assert_ne!(
        q.capability_id(),
        swapped.capability_id(),
        "receipt roles must be ordered"
    );
}

/// RFC falsifier: "compiled experience changes semantic identity without changing
/// its digest". The capability identity binds the semantic subject, and the
/// qualified capability carries it.
#[test]
fn capability_identity_changes_with_semantic_subject() {
    let qualify = |subject: &str| {
        let mut ev = evidence();
        ev.semantic_subject = d(subject);
        let c = candidate_from_execution(&ev).unwrap();
        qualify_candidate(
            &c,
            d("ontology"),
            d("manufacturer"),
            Some(d("admit")),
            Some(d("verify")),
        )
        .unwrap()
    };
    let a = qualify("subject");
    let b = qualify("subject-2");
    assert_eq!(a.semantic_subject(), d("subject"));
    assert_eq!(b.semantic_subject(), d("subject-2"));
    assert_ne!(a.capability_id(), b.capability_id());
    assert_ne!(a.promotion_candidate_id(), b.promotion_candidate_id());
    // every qualified field is the candidate's, never a caller's substitute
    assert_eq!(a.ontology_digest(), d("ontology"));
    assert_eq!(a.manufacturer_digest(), d("manufacturer"));
    assert_eq!(a.admission_receipt(), d("admit"));
    assert_eq!(a.verification_receipt(), d("verify"));
}

#[test]
fn null_receipts_are_not_evidence() {
    let c = candidate_from_execution(&evidence()).unwrap();
    for (admit, verify) in [(Digest::ZERO, d("verify")), (d("admit"), Digest::ZERO)] {
        assert_eq!(
            qualify_candidate(
                &c,
                d("ontology"),
                d("manufacturer"),
                Some(admit),
                Some(verify)
            ),
            Err(ExperienceRefusal::NullReceipt)
        );
    }
}

/// The only route to a candidate is `candidate_from_execution`; a failed execution
/// therefore never reaches `qualify_candidate` (struct-literal construction outside
/// the crate is a compile error, see the `compile_fail` doctest on
/// `PromotionCandidate`).
#[test]
fn failed_execution_has_no_path_to_qualification() {
    let mut ev = evidence();
    ev.execution_succeeded = false;
    let outcome = candidate_from_execution(&ev).and_then(|c| {
        qualify_candidate(
            &c,
            d("ontology"),
            d("manufacturer"),
            Some(d("admit")),
            Some(d("verify")),
        )
    });
    assert_eq!(outcome, Err(ExperienceRefusal::ExecutionNotSuccessful));
}
