//! replay_law — Chicago TDD conformance and replay law tests.
//!
//! Doctrine (Van der Aalst Constitution):
//! If the code says it worked but the event log cannot prove a lawful process
//! happened, then it did not work.

use bcinr_powl::receipt::causal_receipt::OcelCausalReceipt;
use bcinr_powl::receipt::conformance::{
    ConformanceDimension, ConformanceMetrics, ConformancePredicate,
};
use bcinr_powl::receipt::denial::DenialPolarity;
use bcinr_powl::receipt::ocel_emit::OcelEmitArena;
use bcinr_powl::receipt::pm_bridge::frames_to_ocel2_json;
use bcinr_powl::receipt::replay::{PowlReplayFrame, PowlReplayVerifier, ReplayViolation};

// ── Helper ─────────────────────────────────────────────────────────────────────

fn pframe(
    node_id: u32,
    node_bit: u64,
    required: u64,
    produces: u64,
    activity: &str,
) -> PowlReplayFrame {
    PowlReplayFrame {
        node_id,
        node_bit,
        required_tokens: required,
        produces_tokens: produces,
        activity: activity.to_string(),
        ts_ns: node_id as u64 * 1_000_000_000,
        object_ids: vec![format!("obj-{node_id}")],
    }
}

// ── valid_trace_achieves_perfect_fitness ──────────────────────────────────────

#[test]
fn valid_trace_achieves_perfect_fitness() {
    // A three-node linear process: A → B → C.
    // Token starts at node A (bit 0x1). Each node consumes its input token and
    // produces the successor's token.
    let mut v = PowlReplayVerifier::new(0x1);
    let frames = [
        pframe(0, 0x1, 0x1, 0x2, "A"),
        pframe(1, 0x2, 0x2, 0x4, "B"),
        pframe(2, 0x4, 0x4, 0x0, "C"),
    ];
    for f in &frames {
        assert!(
            v.replay_frame(f).is_ok(),
            "law violated: valid frame {} must replay without error",
            f.node_id
        );
    }
    let m = v.finalize();
    assert_eq!(
        m.fitness, 0x0001_0000,
        "law violated: perfect trace must yield fitness == 1.0 (Q16.16 = 0x0001_0000); got {:#010x}",
        m.fitness
    );
}

// ── out_of_order_frame_is_rejected ────────────────────────────────────────────

#[test]
fn out_of_order_frame_is_rejected() {
    // Van der Aalst: a token must be present before a node fires.
    // Entry token = bit 0x1 (node A). Replaying node B (requires token 0x2)
    // before A fires is a log ordering violation.
    let mut v = PowlReplayVerifier::new(0x1);
    let b = pframe(1, 0x2, 0x2, 0x4, "B");
    assert_eq!(
        v.replay_frame(&b),
        Err(ReplayViolation::TokenNotEnabled { node_id: 1 }),
        "law violated: replaying an unenabled frame must yield TokenNotEnabled"
    );
}

#[test]
fn out_of_order_skipping_to_third_node_is_rejected() {
    let mut v = PowlReplayVerifier::new(0x1);
    let c = pframe(2, 0x4, 0x4, 0x0, "C");
    assert!(
        matches!(
            v.replay_frame(&c),
            Err(ReplayViolation::TokenNotEnabled { .. })
        ),
        "law violated: firing the third node without the first two is illegal"
    );
}

// ── strict_predicate_fails_on_low_precision ───────────────────────────────────

#[test]
fn strict_predicate_fails_on_low_precision() {
    // STRICT demands min_precision = 0x0001_0000 (≈ 1.0).
    // Supplying precision = 0x0000_7000 (≈ 0.44) must produce a Precision violation.
    let m = ConformanceMetrics {
        fitness: 0x0001_0000,
        precision: 0x0000_7000,
        generalization: 0x0000_8000,
        simplicity: 0x0000_8000,
    };
    let result = ConformancePredicate::STRICT.check(&m);
    assert!(
        result.is_err(),
        "law violated: STRICT must reject low precision"
    );
    let violation = result.unwrap_err();
    assert_eq!(
        violation.dim,
        ConformanceDimension::Precision,
        "law violated: the failing dimension must be Precision, not {:?}",
        violation.dim
    );
}

#[test]
fn strict_predicate_passes_at_exact_threshold() {
    let m = ConformanceMetrics {
        fitness: 0x0001_0000,
        precision: 0x0001_0000,
        generalization: 0x0000_8000,
        simplicity: 0x0000_8000,
    };
    assert!(
        ConformancePredicate::STRICT.check(&m).is_ok(),
        "law violated: metrics at STRICT threshold must pass"
    );
}

// ── denied_frames_produce_distinct_fired_mask ────────────────────────────────

#[test]
fn denied_frames_produce_distinct_fired_mask() {
    // Van der Aalst: different denial lanes = different trace variants in the DFG.
    // SLA_BREACH lives in lane 2 (bit 2 of fired_mask).
    // AUTHORIZATION_DENIED lives in lane 3 (bit 3 of fired_mask).
    // They must occupy distinct bits so the DFG sees two distinct trace variants.
    let sla_mask = DenialPolarity::SLA_BREACH.to_fired_mask();
    let auth_mask = DenialPolarity::AUTHORIZATION_DENIED.to_fired_mask();

    assert_ne!(
        sla_mask, auth_mask,
        "law violated: SLA_BREACH and AUTHORIZATION_DENIED must map to different fired_mask bits"
    );

    // Verify exact bit positions (lane 2 → bit 2, lane 3 → bit 3).
    assert_eq!(sla_mask, 1 << 2, "SLA_BREACH must scatter to bit 2");
    assert_eq!(
        auth_mask,
        1 << 3,
        "AUTHORIZATION_DENIED must scatter to bit 3"
    );

    // Composed denial must set BOTH bits — two distinct trace variants visible.
    let composed = DenialPolarity::SLA_BREACH
        .compose(DenialPolarity::AUTHORIZATION_DENIED)
        .to_fired_mask();
    assert_eq!(
        composed,
        (1 << 2) | (1 << 3),
        "composed denial must set both bits; distinct variants in DFG"
    );
}

#[test]
fn admitted_denial_produces_zero_fired_mask() {
    assert_eq!(
        DenialPolarity::ADMITTED.to_fired_mask(),
        0,
        "ADMITTED must not activate any fired_mask bit"
    );
}

#[test]
fn all_denial_constants_have_distinct_fired_mask_bits() {
    let constants = [
        ("PRECONDITION_FAILED", DenialPolarity::PRECONDITION_FAILED),
        ("SLA_BREACH", DenialPolarity::SLA_BREACH),
        ("AUTHORIZATION_DENIED", DenialPolarity::AUTHORIZATION_DENIED),
        ("RESOURCE_EXHAUSTED", DenialPolarity::RESOURCE_EXHAUSTED),
        (
            "OBJECT_LIFECYCLE_VIOLATION",
            DenialPolarity::OBJECT_LIFECYCLE_VIOLATION,
        ),
        (
            "CONFORMANCE_GATE_FAILED",
            DenialPolarity::CONFORMANCE_GATE_FAILED,
        ),
        ("WATCHDOG_DRAINED", DenialPolarity::WATCHDOG_DRAINED),
    ];
    let mut seen: u64 = 0;
    for (name, dp) in &constants {
        let bit = dp.to_fired_mask();
        assert_ne!(bit, 0, "{name} must produce a non-zero fired_mask");
        assert_eq!(
            bit.count_ones(),
            1,
            "{name} must scatter to exactly one bit"
        );
        assert_eq!(
            seen & bit,
            0,
            "{name} bit collides with a previously seen constant"
        );
        seen |= bit;
    }
}

// ── pm_bridge_produces_ocel_2_0_json ─────────────────────────────────────────

#[test]
fn pm_bridge_produces_ocel_2_0_json() {
    let frames = vec![
        pframe(0, 0x1, 0x1, 0x2, "manufacture"),
        pframe(1, 0x2, 0x2, 0x4, "validate"),
        pframe(2, 0x4, 0x4, 0x0, "release"),
    ];

    let json = frames_to_ocel2_json(&frames);

    // Top-level OCEL 2.0 required keys.
    assert_eq!(
        json["ocel:type"], "powl-causal-trace",
        "ocel:type must identify the trace type"
    );
    assert!(
        json.get("ocel:attribute-names").is_some(),
        "OCEL 2.0 JSON must include ocel:attribute-names"
    );

    let events = json["ocel:events"]
        .as_object()
        .expect("ocel:events must be a JSON object");
    assert_eq!(events.len(), 3, "event count must match frame count");

    for (event_id, event) in events {
        assert!(
            event.get("ocel:type").is_some(),
            "event {event_id} missing ocel:type activity label"
        );
        assert!(
            event.get("ocel:timestamp").is_some(),
            "event {event_id} missing ocel:timestamp"
        );
        assert!(
            event.get("ocel:omap").is_some(),
            "event {event_id} missing ocel:omap (E2O links)"
        );
    }

    let objects = json["ocel:objects"]
        .as_object()
        .expect("ocel:objects must be a JSON object");
    // Three distinct object ids (one per frame: "obj-0", "obj-1", "obj-2").
    assert_eq!(
        objects.len(),
        3,
        "ocel:objects must list all distinct objects"
    );
}

#[test]
fn pm_bridge_deduplicates_objects_across_frames() {
    let mut f0 = pframe(0, 0x1, 0x1, 0x2, "A");
    f0.object_ids = vec!["case-1".into(), "item-42".into()];
    let mut f1 = pframe(1, 0x2, 0x2, 0x0, "B");
    f1.object_ids = vec!["case-1".into()]; // shared object

    let json = frames_to_ocel2_json(&[f0, f1]);
    let objects = json["ocel:objects"].as_object().unwrap();
    assert_eq!(
        objects.len(),
        2,
        "duplicate object ids must be deduplicated in ocel:objects"
    );
    assert!(objects.contains_key("case-1"));
    assert!(objects.contains_key("item-42"));
}

#[test]
fn pm_bridge_empty_trace_is_valid_ocel() {
    let json = frames_to_ocel2_json(&[]);
    assert_eq!(json["ocel:events"].as_object().unwrap().len(), 0);
    assert_eq!(json["ocel:objects"].as_object().unwrap().len(), 0);
}

// ── Causal receipt chain law ───────────────────────────────────────────────────

#[test]
fn causal_receipt_hash_advances_monotonically() {
    let mut arena = OcelEmitArena::new();
    let mut receipt = OcelCausalReceipt::genesis([0u8; 32]);
    let mut hashes: std::collections::HashSet<[u8; 32]> = std::collections::HashSet::new();
    hashes.insert(receipt.chain_hash);

    for i in 0..10u64 {
        let frame = arena.emit(i, 0, &[], DenialPolarity::ADMITTED, 0);
        receipt.chain(frame);
        assert!(
            hashes.insert(receipt.chain_hash),
            "law violated: chain hash must change after each frame (duplicate at frame {i})"
        );
    }
    assert_eq!(receipt.frame_count, 10);
}
