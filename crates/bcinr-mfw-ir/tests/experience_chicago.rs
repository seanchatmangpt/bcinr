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
    assert_eq!(a.authority, ExperienceAuthority::None);
    assert_eq!(a.source_execution_receipt, d("exec-receipt"));
}

#[test]
fn every_semantic_input_changes_candidate_identity() {
    let base = candidate_from_execution(&evidence()).unwrap().candidate_id;
    let mutations: [fn(&mut ExperienceEvidence); 4] = [
        |e| e.semantic_subject = d("subject-2"),
        |e| e.execution_receipt = d("exec-receipt-2"),
        |e| e.ontology_digest = d("ontology-2"),
        |e| e.manufacturer_digest = d("manufacturer-2"),
    ];
    for mutate in mutations {
        let mut ev = evidence();
        mutate(&mut ev);
        assert_ne!(candidate_from_execution(&ev).unwrap().candidate_id, base);
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
    assert_eq!(q.authority, ExperienceAuthority::None);
    assert_eq!(q.promotion_candidate_id, c.candidate_id);
    assert_ne!(q.capability_id, c.candidate_id);
    let swapped = qualify_candidate(
        &c,
        d("ontology"),
        d("manufacturer"),
        Some(d("verify")),
        Some(d("admit")),
    )
    .unwrap();
    assert_ne!(
        q.capability_id, swapped.capability_id,
        "receipt roles must be ordered"
    );
}
