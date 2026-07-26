//! Chicago-TDD integration tests for bcinr-powl.
//!
//! The suite exercises the real compiler, scheduler, temporal conformance validator,
//! deterministic trace receipt, Chicago diagnostic collector, and an independent OCEL 2.0
//! importer/semantic validator.
//!
//! Run with:
//!   cargo test -p bcinr-powl --test chicago_tdd_integration --features std

#![cfg(feature = "std")]

use bcinr_powl::compiler::{compile_powl, PowlAstNode};
use bcinr_powl::ocel::{ConformanceResult, OcelEvent, OcelLog};
use bcinr_powl::scheduler::{scheduler_tick, PowlRunState};
use bcinr_powl::tape::OpKind;
use chicago_tdd_tools::core::governance::{
    Diagnostic, DiagnosticCategory, DiagnosticCode, DiagnosticSink, RunSummary, Severity,
};
use chicago_tdd_tools::observability::ocel::wasm4pm::seal_run;
use chicago_tdd_tools::observability::ocel::OcelCollector;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

fn emit_event(
    sink: &OcelCollector,
    collector_run_id: &str,
    event: &OcelEvent,
    severity: Severity,
    elapsed_ns: u64,
) {
    let mut context = HashMap::new();
    context.insert("event_id", serde_json::json!(event.event_id));
    context.insert("activity", serde_json::json!(event.activity));
    context.insert("timestamp", serde_json::json!(event.timestamp));
    context.insert("powl_run_id", serde_json::json!(event.run_id));
    context.insert("op_idx", serde_json::json!(event.op_idx));
    context.insert(
        "op_trace",
        serde_json::json!(format!("{:#018x}", event.op_trace)),
    );
    context.insert("kind_tag", serde_json::json!(event.kind_tag));

    sink.emit(Diagnostic {
        code: DiagnosticCode::new("POWL", DiagnosticCategory::Conformance, 1),
        category: DiagnosticCategory::Conformance,
        run_id: collector_run_id.to_string(),
        agent_id: None,
        location: None,
        message: format!(
            "{} run={} op={} trace={:#018x}",
            event.activity, event.run_id, event.op_idx, event.op_trace
        ),
        severity,
        source_module: "chicago_tdd_integration",
        context,
        elapsed_ns,
    })
    .expect("POWL trace diagnostic must be admitted");
}

fn mirror_trace(
    sink: &OcelCollector,
    collector_run_id: &str,
    log: &OcelLog,
    severity: Severity,
    elapsed_start: u64,
) -> usize {
    for (offset, event) in log.events().iter().enumerate() {
        emit_event(
            sink,
            collector_run_id,
            event,
            severity,
            elapsed_start + offset as u64,
        );
    }
    log.events().len()
}

fn emit_summary(
    sink: &OcelCollector,
    run_id: &str,
    severity: Severity,
    message: &str,
    elapsed_ns: u64,
) {
    sink.emit(Diagnostic {
        code: DiagnosticCode::new("POWL", DiagnosticCategory::Conformance, 2),
        category: DiagnosticCategory::Conformance,
        run_id: run_id.to_string(),
        agent_id: None,
        location: None,
        message: message.to_string(),
        severity,
        source_module: "chicago_tdd_integration",
        context: HashMap::new(),
        elapsed_ns,
    })
    .expect("POWL conformance summary must be admitted");
}

fn make_run_summary(run_id: &str, total_diagnostics: usize, andon_count: usize) -> RunSummary {
    RunSummary {
        run_id: run_id.to_string(),
        evaluated: 1,
        admitted: usize::from(andon_count == 0),
        p_admitted: if andon_count == 0 { 1.0 } else { 0.0 },
        andon_count,
        warning_count: 0,
        dominant_category: Some(DiagnosticCategory::Conformance),
        total_diagnostics,
        category_counts: HashMap::new(),
    }
}

fn execute(
    ast: &PowlAstNode<'_>,
    run_id: u64,
) -> (bcinr_powl::tape::PowlTape, PowlRunState, OcelLog, u64) {
    let tape = compile_powl(ast).expect("POWL model must compile");
    let mut state = PowlRunState::new(&tape);
    let mut log = OcelLog::new();
    let mut op_trace = 0u64;

    for _ in 0..128 {
        if state.check_mask == 0 {
            break;
        }
        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;
        while bits != 0 {
            let op_idx = bits.trailing_zeros();
            bits &= bits - 1;
            log.record_op_fired(run_id, op_idx, 0).unwrap();
            op_trace |= 1u64 << op_idx;
        }
    }
    assert_eq!(state.check_mask, 0, "bounded scheduler must complete");
    log.record_run_sealed(run_id, op_trace).unwrap();
    (tape, state, log, op_trace)
}

fn assert_independent_ocel_conformance(path: &Path) {
    let imported = ::ocel::io::json::read_path(path)
        .expect("independent OCEL implementation must import the exported log");
    imported
        .validate()
        .unwrap_or_else(|violations| panic!("independent OCEL validation failed: {violations:?}"));
}

#[test]
fn powl_linear_chain_conformance_is_trace_receipted() {
    let ast = PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("setup"),
        PowlAstNode::Atom("process"),
        PowlAstNode::Atom("teardown"),
    ]);
    let (tape, _state, log, _) = execute(&ast, 42);
    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms
    );

    let receipt = log.seal_receipt();
    assert_eq!(receipt.log().events(), log.events());
    assert_eq!(receipt.event_count(), log.events().len());

    std::fs::create_dir_all("target").unwrap();
    let path = PathBuf::from("target/bcinr-powl-chicago-tdd-linear-chain.ocel.json");
    let collector = OcelCollector::new(Some(path));
    let mirrored = mirror_trace(&collector, "powl-linear-001", &log, Severity::Info, 1_000);
    emit_summary(
        &collector,
        "powl-linear-001",
        Severity::Info,
        "temporal predecessor law admitted",
        1_000 + mirrored as u64,
    );

    let (receipted, digest) =
        seal_run(&collector, "powl-linear-001".to_string()).expect("seal_run must succeed");
    assert_eq!(digest.len(), 64);
    assert!(digest.chars().all(|c| c.is_ascii_hexdigit()));
    assert!(receipted.inner().events.len() >= mirrored);
    collector
        .close(make_run_summary("powl-linear-001", mirrored + 1, 0))
        .expect("collector close must succeed");
}

#[test]
fn powl_temporal_predecessor_inversion_is_refused_and_receipted() {
    let tape = compile_powl(&PowlAstNode::Sequence(vec![
        PowlAstNode::Atom("precondition"),
        PowlAstNode::Atom("action"),
    ]))
    .unwrap();
    let mut log = OcelLog::new();
    log.record_op_fired(99, 1, 0).unwrap();
    log.record_op_fired(99, 0, 0).unwrap();
    log.record_run_sealed(99, 0b11).unwrap();

    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Violation {
            run_id: 99,
            op_idx: 1,
            missing_pred_mask: 0b01,
        }
    );

    let receipt = log.seal_receipt();
    assert_eq!(receipt.log().events()[0].op_idx, 1);
    assert_eq!(receipt.log().events()[1].op_idx, 0);

    let collector = OcelCollector::new(None);
    let mirrored = mirror_trace(
        &collector,
        "powl-violation-001",
        &log,
        Severity::Andon,
        2_000,
    );
    emit_summary(
        &collector,
        "powl-violation-001",
        Severity::Andon,
        "temporal predecessor inversion refused",
        2_000 + mirrored as u64,
    );
    let (receipted, digest) = seal_run(&collector, "powl-violation-001".to_string())
        .expect("violation trace must still be receiptable");
    assert_eq!(digest.len(), 64);
    assert!(receipted.inner().events.len() >= mirrored);
}

#[test]
fn powl_xor_executes_exactly_one_branch_and_exports_independently_valid_ocel() {
    let ast = PowlAstNode::XorChoice(vec![
        PowlAstNode::Atom("branch_a"),
        PowlAstNode::Atom("branch_b"),
        PowlAstNode::Atom("branch_c"),
    ]);
    let (tape, state, log, op_trace) = execute(&ast, 7);
    assert_eq!(
        log.validate_against_tape(&tape),
        ConformanceResult::Conforms
    );

    let dispatch = tape.ops[..tape.len as usize]
        .iter()
        .find(|op| op.kind == OpKind::XorDispatch)
        .expect("compiled XOR must contain a dispatch operation");
    let fired_branches = op_trace & dispatch.branch_mask;
    assert_eq!(
        fired_branches.count_ones(),
        1,
        "exactly one XOR branch may fire"
    );
    assert_eq!(state.choice_taken, fired_branches);

    let json = log
        .to_ocel_json()
        .expect("OCEL 2.0 JSON export must succeed");
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    for key in ["objectTypes", "eventTypes", "objects", "events"] {
        assert!(
            parsed.get(key).is_some(),
            "missing OCEL top-level key {key}"
        );
    }

    let objects = parsed["objects"].as_array().unwrap();
    let declared_ids: std::collections::HashSet<&str> = objects
        .iter()
        .filter_map(|object| object.get("id").and_then(|id| id.as_str()))
        .collect();
    for event in parsed["events"].as_array().unwrap() {
        assert!(event.get("time").is_some(), "OCEL event is missing time");
        for relation in event
            .get("relationships")
            .and_then(|value| value.as_array())
            .into_iter()
            .flatten()
        {
            let object_id = relation["objectId"].as_str().unwrap();
            assert!(declared_ids.contains(object_id));
        }
    }

    std::fs::create_dir_all("target").unwrap();
    let export_path = PathBuf::from("target/bcinr-powl-chicago-tdd-xor-workflow.jsonocel");
    std::fs::write(&export_path, &json).unwrap();
    assert_independent_ocel_conformance(&export_path);

    let collector = OcelCollector::new(Some(PathBuf::from(
        "target/bcinr-powl-chicago-tdd-xor-collector.ocel.json",
    )));
    let mirrored = mirror_trace(&collector, "powl-xor-001", &log, Severity::Info, 5_000);
    emit_summary(
        &collector,
        "powl-xor-001",
        Severity::Info,
        "XOR exact-one and independent OCEL validation admitted",
        5_000 + mirrored as u64,
    );
    let (receipted, _) =
        seal_run(&collector, "powl-xor-001".to_string()).expect("XOR trace must seal");
    assert!(receipted.inner().events.len() >= mirrored);
    collector
        .close(make_run_summary("powl-xor-001", mirrored + 1, 0))
        .unwrap();
}

#[test]
fn deterministic_trace_receipts_replay_identically() {
    let build = || {
        let mut log = OcelLog::new();
        log.record_op_fired(123, 0, 7).unwrap();
        log.record_op_fired(123, 1, 8).unwrap();
        log.record_run_sealed(123, 0b11).unwrap();
        log
    };
    let first = build().seal_receipt();
    let second = build().seal_receipt();
    assert_eq!(first.digest(), second.digest());
    assert_eq!(first.log().events(), second.log().events());
}

#[test]
fn deterministic_trace_receipt_binds_each_mutation_dimension() {
    fn digest(
        run_id: u64,
        first_op: u32,
        first_kind: u8,
        second_op: Option<u32>,
        seal: u64,
    ) -> [u8; 32] {
        let mut log = OcelLog::new();
        log.record_op_fired(run_id, first_op, first_kind).unwrap();
        if let Some(op_idx) = second_op {
            log.record_op_fired(run_id, op_idx, 9).unwrap();
        }
        log.record_run_sealed(run_id, seal).unwrap();
        log.seal_receipt().digest()
    }

    let baseline = digest(55, 0, 7, None, 0b1);
    let mutations = [
        digest(56, 0, 7, None, 0b1),     // run identity
        digest(55, 1, 7, None, 0b1),     // operation identity
        digest(55, 0, 8, None, 0b1),     // operation kind
        digest(55, 0, 7, Some(1), 0b11), // event count and ordered content
        digest(55, 0, 7, None, 0b11),    // declared seal trace only
    ];

    for (index, mutated) in mutations.into_iter().enumerate() {
        assert_ne!(
            baseline, mutated,
            "mutation dimension {index} escaped the receipt"
        );
    }
}
