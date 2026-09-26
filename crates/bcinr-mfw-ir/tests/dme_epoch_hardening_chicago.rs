//! Adversarial / boundary falsifiers for recursive DME epoch closure (A2A-2606).
//!
//! Chicago style: real `DmeEpoch`, real `DescentMeter`, real `advance_epoch`;
//! assertions on returned epochs, digests and typed refusals. No test doubles.

use bcinr_mfw_ir::{
    advance_epoch, DescentMeter, Digest, DmeEpoch, EpochAdvance, EpochAuthority, EpochRefusal,
    ReceiptFeedback,
};

fn d(value: &str) -> Digest {
    Digest::hash(value.as_bytes())
}

fn feedback(parent: &DmeEpoch, receipt: &str, residuals: Vec<Digest>) -> ReceiptFeedback {
    ReceiptFeedback {
        receipt_digest: d(receipt),
        observed_ontology_digest: parent.ontology_digest,
        residual_obligations: residuals,
        authority: EpochAuthority::None,
    }
}

fn successor(result: Result<EpochAdvance, EpochRefusal>) -> DmeEpoch {
    match result {
        Ok(EpochAdvance::Successor(child)) => child,
        other => panic!("expected successor, got {other:?}"),
    }
}

// ---- reordering -----------------------------------------------------------

#[test]
fn root_identity_is_independent_of_residual_order() {
    let a = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let b = DmeEpoch::root(d("o"), vec![d("c"), d("a"), d("b")]).unwrap();
    assert_eq!(a.epoch_id, b.epoch_id);
    assert_eq!(a, b);
}

#[test]
fn successor_identity_is_independent_of_feedback_order() {
    let parent = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let mut m1 = DescentMeter::new(8);
    let mut m2 = DescentMeter::new(8);
    let x = successor(advance_epoch(
        &parent,
        &feedback(&parent, "r", vec![d("a"), d("c")]),
        &mut m1,
    ));
    let y = successor(advance_epoch(
        &parent,
        &feedback(&parent, "r", vec![d("c"), d("a")]),
        &mut m2,
    ));
    assert_eq!(x, y);
}

// ---- malformed / minted obligations ----------------------------------------

#[test]
fn feedback_cannot_substitute_fresh_obligations_for_discharged_ones() {
    let parent = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let mut meter = DescentMeter::new(8);
    let forged = feedback(&parent, "r", vec![d("minted")]);
    assert_eq!(
        advance_epoch(&parent, &forged, &mut meter),
        Err(EpochRefusal::ResidualNotInParent)
    );
    assert_eq!(meter.depth, 0, "refusal must not consume descent budget");

    let mixed = feedback(&parent, "r", vec![d("a"), d("minted")]);
    assert_eq!(
        advance_epoch(&parent, &mixed, &mut meter),
        Err(EpochRefusal::ResidualNotInParent)
    );
}

#[test]
fn duplicate_residuals_are_refused_at_root_and_in_feedback() {
    assert_eq!(
        DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("a")]),
        Err(EpochRefusal::DuplicateResidual)
    );
    let parent = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let mut meter = DescentMeter::new(8);
    let dup = feedback(&parent, "r", vec![d("a"), d("a")]);
    assert_eq!(
        advance_epoch(&parent, &dup, &mut meter),
        Err(EpochRefusal::DuplicateResidual)
    );
}

// ---- stale subject / stale meter ------------------------------------------

#[test]
fn ontology_drift_is_refused_even_on_the_closure_path() {
    let parent = DmeEpoch::root(d("o"), vec![d("a")]).unwrap();
    let mut meter = DescentMeter::new(8);
    let mut close = feedback(&parent, "r", vec![]);
    close.observed_ontology_digest = d("o-prime");
    assert_eq!(
        advance_epoch(&parent, &close, &mut meter),
        Err(EpochRefusal::OntologyIdentityChanged)
    );
}

#[test]
fn a_fresh_meter_cannot_re_mint_depth_for_a_deeper_epoch() {
    let root = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let mut meter = DescentMeter::new(8);
    let child = successor(advance_epoch(
        &root,
        &feedback(&root, "r1", vec![d("a"), d("b")]),
        &mut meter,
    ));
    assert_eq!(child.depth, 1);

    let mut fresh = DescentMeter::new(8);
    assert_eq!(
        advance_epoch(&child, &feedback(&child, "r2", vec![d("a")]), &mut fresh),
        Err(EpochRefusal::StaleDescentMeter)
    );
    assert_eq!(fresh.depth, 0);

    let grandchild = successor(advance_epoch(
        &child,
        &feedback(&child, "r2", vec![d("a")]),
        &mut meter,
    ));
    assert_eq!(grandchild.depth, 2);
    assert_eq!(grandchild.parent_epoch_id, Some(child.epoch_id));
}

// ---- duplicate delivery / replay ------------------------------------------

#[test]
fn duplicate_delivery_of_the_same_feedback_to_the_child_is_refused() {
    let root = DmeEpoch::root(d("o"), vec![d("a"), d("b"), d("c")]).unwrap();
    let mut meter = DescentMeter::new(8);
    let fb = feedback(&root, "r1", vec![d("a"), d("b")]);
    let child = successor(advance_epoch(&root, &fb, &mut meter));
    assert_eq!(
        advance_epoch(&child, &fb, &mut meter),
        Err(EpochRefusal::NonDescendingResidual)
    );
    assert_eq!(meter.depth, 1);
}

#[test]
fn receipt_identity_is_bound_into_successor_and_closure_identity() {
    let root = DmeEpoch::root(d("o"), vec![d("a"), d("b")]).unwrap();
    let x = successor(advance_epoch(
        &root,
        &feedback(&root, "r1", vec![d("a")]),
        &mut DescentMeter::new(8),
    ));
    let y = successor(advance_epoch(
        &root,
        &feedback(&root, "r2", vec![d("a")]),
        &mut DescentMeter::new(8),
    ));
    assert_ne!(x.epoch_id, y.epoch_id);

    let close = |receipt: &str| match advance_epoch(
        &root,
        &feedback(&root, receipt, vec![]),
        &mut DescentMeter::new(8),
    ) {
        Ok(EpochAdvance::Closed {
            closure_digest,
            authority,
            ..
        }) => {
            assert_eq!(authority, EpochAuthority::None);
            closure_digest
        }
        other => panic!("expected closure, got {other:?}"),
    };
    assert_ne!(close("c1"), close("c2"));
    assert_eq!(close("c1"), close("c1"));
}

// ---- termination ------------------------------------------------------------

#[test]
fn any_admitted_chain_closes_within_the_root_residual_count() {
    let n = 16usize;
    let residuals: Vec<Digest> = (0..n).map(|i| d(&format!("obl-{i}"))).collect();
    let mut epoch = DmeEpoch::root(d("o"), residuals.clone()).unwrap();
    let mut meter = DescentMeter::new(n);
    let mut steps = 0usize;
    loop {
        let keep = epoch.residual_obligations.len().saturating_sub(1);
        let next: Vec<Digest> = epoch
            .residual_obligations
            .iter()
            .take(keep)
            .copied()
            .collect();
        let fb = feedback(&epoch, &format!("r{steps}"), next);
        steps += 1;
        match advance_epoch(&epoch, &fb, &mut meter).unwrap() {
            EpochAdvance::Closed { .. } => break,
            EpochAdvance::Successor(child) => {
                assert!(child.residual_obligations.len() < epoch.residual_obligations.len());
                assert_eq!(child.depth, epoch.depth + 1);
                epoch = child;
            }
        }
        assert!(steps <= n, "chain did not terminate within {n} steps");
    }
    assert_eq!(steps, n);
    assert_eq!(meter.depth, n - 1);
}
