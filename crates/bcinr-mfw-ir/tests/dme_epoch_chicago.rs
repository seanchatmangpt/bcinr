use bcinr_mfw_ir::{
    advance_epoch, DescentMeter, Digest, DmeEpoch, EpochAdvance, EpochAuthority, EpochRefusal,
    ReceiptFeedback,
};

fn d(value: &str) -> Digest { Digest::hash(value.as_bytes()) }

fn root() -> DmeEpoch {
    DmeEpoch::root(d("ontology-v1"), vec![d("a"), d("b"), d("c")]).unwrap()
}

#[test]
fn closed_plan_emits_no_successor_epoch() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-close"),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: vec![],
        authority: EpochAuthority::None,
    };
    let mut meter = DescentMeter::new(8);
    let result = advance_epoch(&parent, &feedback, &mut meter).unwrap();
    assert!(matches!(result, EpochAdvance::Closed { authority: EpochAuthority::None, .. }));
    assert_eq!(meter.depth, 0);
}

#[test]
fn non_descending_residual_loop_is_refused_before_recursion() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-loop"),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: parent.residual_obligations.clone(),
        authority: EpochAuthority::None,
    };
    let mut meter = DescentMeter::new(8);
    assert_eq!(advance_epoch(&parent, &feedback, &mut meter), Err(EpochRefusal::NonDescendingResidual));
    assert_eq!(meter.depth, 0);
}

#[test]
fn child_cannot_change_parent_ontology_identity() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-drift"),
        observed_ontology_digest: d("ontology-v2"),
        residual_obligations: vec![d("a")],
        authority: EpochAuthority::None,
    };
    let mut meter = DescentMeter::new(8);
    assert_eq!(advance_epoch(&parent, &feedback, &mut meter), Err(EpochRefusal::OntologyIdentityChanged));
}

#[test]
fn receipt_feedback_changes_residuals_but_never_authority() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-child"),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: vec![d("a"), d("b")],
        authority: EpochAuthority::None,
    };
    let mut meter = DescentMeter::new(8);
    let result = advance_epoch(&parent, &feedback, &mut meter).unwrap();
    let EpochAdvance::Successor(child) = result else { panic!("expected successor") };
    assert_eq!(child.parent_epoch_id, Some(parent.epoch_id));
    assert_eq!(child.authority, EpochAuthority::None);
    assert_eq!(child.residual_obligations.len(), 2);
    assert_eq!(child.depth, 1);
}

#[test]
fn replayed_feedback_has_identical_semantic_identity_not_a_second_epoch_identity() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-replay"),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: vec![d("a")],
        authority: EpochAuthority::None,
    };
    let mut first_meter = DescentMeter::new(8);
    let mut replay_meter = DescentMeter::new(8);
    let first = advance_epoch(&parent, &feedback, &mut first_meter).unwrap();
    let replay = advance_epoch(&parent, &feedback, &mut replay_meter).unwrap();
    assert_eq!(first, replay);
}

#[test]
fn descent_budget_refuses_child_manufacture() {
    let parent = root();
    let feedback = ReceiptFeedback {
        receipt_digest: d("receipt-bound"),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: vec![d("a")],
        authority: EpochAuthority::None,
    };
    let mut meter = DescentMeter::new(0);
    assert_eq!(advance_epoch(&parent, &feedback, &mut meter), Err(EpochRefusal::DescentBoundHit));
}
