#!/usr/bin/env python3
"""Apply the final observed v26.7.26 source repairs.

This is a temporary, fail-closed bootstrap. It edits only the files named in
EXPECTED_FILES and refuses unless every postcondition is present afterward.
"""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EXPECTED_FILES = {
    "crates/bcinr-cmca/tests/usecase_ml_fairness_audit.rs",
    "crates/bcinr-cmca/tests/usecase_radiation_hardened.rs",
    "crates/bcinr-powl/src/ocel.rs",
    "crates/bcinr-powl/tests/chaos_harness.rs",
    "crates/bcinr-powl/tests/chaos_scenarios.rs",
    "crates/bcinr-powl/tests/chicago_tdd_integration.rs",
    "crates/bcinr-powl/tests/usecase_compliance_audit.rs",
    "crates/bcinr-powl/tests/usecase_swarm_02_parallel_independent_workers.rs",
    "scripts/generate_v26_7_26_report.sh",
}


def edit(rel: str, transform) -> str:
    file = ROOT / rel
    before = file.read_text(encoding="utf-8")
    after = transform(before)
    if before == after:
        print(f"UNCHANGED {rel}")
    else:
        file.write_text(after, encoding="utf-8")
        print(f"CHANGED {rel}")
    return after


def require(rel: str, text: str, needle: str, count: int | None = None) -> None:
    actual = text.count(needle)
    if actual == 0 or (count is not None and actual != count):
        raise SystemExit(
            f"REPAIR_REFUSED: {rel}: {needle!r} count={actual}, expected={count or '>=1'}"
        )
    print(f"ADMITTED {rel}: {needle!r} count={actual}")


def repair_radiation(text: str) -> str:
    text = re.sub(
        r"    for i in 1\.\.results\.len\(\) \{\n        for j in 0\.\.N \{\n            assert_eq!\(\n                results\[i\]\[j\], results\[0\]\[j\],\n                (\"allocation[^\n]+\")\n            \);\n        \}\n    \}",
        r"    for result in results.iter().skip(1) {\n        for (value, baseline) in result.iter().zip(&results[0]) {\n            assert_eq!(\n                value, baseline,\n                \1\n            );\n        }\n    }",
        text,
        count=1,
    )
    text = re.sub(
        r"        for j in 0\.\.N \{\n            assert_eq!\(\n                repeat\[j\], baseline\[j\],\n                (\"identical round[^\n]+\")\n            \);\n        \}",
        r"        for (value, baseline_value) in repeat.iter().zip(&baseline) {\n            assert_eq!(\n                value, baseline_value,\n                \1\n            );\n        }",
        text,
        count=1,
    )
    return text


def repair_fairness(text: str) -> str:
    text = re.sub(
        r"        for i in 0\.\.N \{\n            assert!\(\n                alloc\[i\]\.val > 0,\n                (\"round[^\n]+\"),\n                round,\n                i\n            \);\n        \}",
        r"        for (i, allocation) in alloc.iter().enumerate().take(N) {\n            assert!(\n                allocation.val > 0,\n                \1,\n                round,\n                i\n            );\n        }",
        text,
        count=1,
    )
    text = re.sub(
        r"        for i in 0\.\.N \{\n            assert_eq!\(\n                alloc\[i\], baseline\[i\],\n                (\"round[^\n]+\"),\n                round, i\n            \);\n        \}",
        r"        for (i, (allocation, baseline_allocation)) in alloc.iter().zip(&baseline).enumerate() {\n            assert_eq!(\n                allocation, baseline_allocation,\n                \1,\n                round, i\n            );\n        }",
        text,
        count=1,
    )
    text = re.sub(
        r"    for i in 0\.\.N \{\n        assert_eq!\(\n            platform_alloc\[i\], auditor_replay\[i\],\n            (\"auditor[^\n]+\"),\n            i\n        \);\n    \}",
        r"    for (i, (platform, replay)) in platform_alloc.iter().zip(&auditor_replay).enumerate() {\n        assert_eq!(\n            platform, replay,\n            \1,\n            i\n        );\n    }",
        text,
        count=1,
    )
    text = text.replace(
        "let max_idx = (0..N).max_by_key(|&i| alloc[i].val).unwrap();",
        "let max_idx = alloc\n        .iter()\n        .enumerate()\n        .max_by_key(|(_, allocation)| allocation.val)\n        .map(|(index, _)| index)\n        .unwrap();",
        1,
    )
    return text


def repair_compliance(text: str) -> str:
    text = text.replace(
        "let mut op_trace = 0u64;",
        "let mut op_trace = 0u64;\n    let mut ticks = 0u32;",
        1,
    )
    text = text.replace(
        "let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;",
        "ticks += 1;\n        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;",
        1,
    )
    replacements = {
        "log.record_op_fired(run_id, op_idx, 0).unwrap();":
            "log.record_op_fired(run_id, op_idx, ticks, 1).unwrap();",
        "log.record_run_sealed(run_id, op_trace).unwrap();":
            "log.record_run_sealed(run_id, op_trace, ticks).unwrap();",
        "log_tampered.record_op_fired(run_id, 0, 0).unwrap();":
            "log_tampered.record_op_fired(run_id, 0, 0, 1).unwrap();",
        "log_tampered.record_op_fired(run_id, 1, 0).unwrap();":
            "log_tampered.record_op_fired(run_id, 1, 1, 1).unwrap();",
        "log_tampered.record_op_fired(run_id, 99, 0).unwrap();":
            "log_tampered.record_op_fired(run_id, 99, 2, 1).unwrap();",
        "log_tampered.record_run_sealed(run_id, 0b111).unwrap();":
            "log_tampered.record_run_sealed(run_id, 0b111, 3).unwrap();",
        "incomplete_log.record_op_fired(1, 1, 0).unwrap();":
            "incomplete_log.record_op_fired(1, 1, 0, 1).unwrap();",
        "incomplete_log.record_run_sealed(1, 0b10).unwrap();":
            "incomplete_log.record_run_sealed(1, 0b10, 1).unwrap();",
    }
    for old, new in replacements.items():
        text = text.replace(old, new)
    return text


def repair_chicago(text: str) -> str:
    text = text.replace(
        'context.insert("timestamp", serde_json::json!(event.timestamp));',
        'context.insert("start_time", serde_json::json!(event.start_time));\n    context.insert("duration", serde_json::json!(event.duration));',
        1,
    )
    text = text.replace(
        'context.insert("kind_tag", serde_json::json!(event.kind_tag));',
        'context.insert(\n        "event_kind",\n        serde_json::json!(format!("{:?}", event.event_kind)),\n    );',
        1,
    )
    text = text.replace(
        "let mut op_trace = 0u64;",
        "let mut op_trace = 0u64;\n    let mut ticks = 0u32;",
        1,
    )
    text = text.replace(
        "let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;",
        "ticks += 1;\n        let mut bits = scheduler_tick(&tape.ops[..tape.len as usize], &mut state).0;",
        1,
    )
    replacements = {
        "log.record_op_fired(run_id, op_idx, 0).unwrap();":
            "log.record_op_fired(run_id, op_idx, ticks, 1).unwrap();",
        "log.record_run_sealed(run_id, op_trace).unwrap();":
            "log.record_run_sealed(run_id, op_trace, ticks).unwrap();",
        "log.record_op_fired(99, 1, 0).unwrap();":
            "log.record_op_fired(99, 1, 0, 1).unwrap();",
        "log.record_op_fired(99, 0, 0).unwrap();":
            "log.record_op_fired(99, 0, 1, 1).unwrap();",
        "log.record_run_sealed(99, 0b11).unwrap();":
            "log.record_run_sealed(99, 0b11, 2).unwrap();",
        "log.record_op_fired(123, 0, 7).unwrap();":
            "log.record_op_fired(123, 0, 7, 1).unwrap();",
        "log.record_op_fired(123, 1, 8).unwrap();":
            "log.record_op_fired(123, 1, 8, 1).unwrap();",
        "log.record_run_sealed(123, 0b11).unwrap();":
            "log.record_run_sealed(123, 0b11, 9).unwrap();",
        "first_kind: u8,": "first_start_time: u32,",
        "log.record_op_fired(run_id, first_op, first_kind).unwrap();":
            "log.record_op_fired(run_id, first_op, first_start_time, 1)\n            .unwrap();",
        "log.record_op_fired(run_id, op_idx, 9).unwrap();":
            "log.record_op_fired(run_id, op_idx, 9, 1).unwrap();",
        "log.record_run_sealed(run_id, seal).unwrap();":
            "log.record_run_sealed(run_id, seal, 10).unwrap();",
        ".record_op_fired(run_id, candidate_idx as u32, i as u8)":
            ".record_op_fired(run_id, candidate_idx as u32, i as u32, 1)",
        ".record_run_sealed(run_id, op_trace)":
            ".record_run_sealed(run_id, op_trace, expected_selections.len() as u32)",
        "emit_conformance(": "emit_summary(",
        "// operation kind": "// operation start time",
    }
    for old, new in replacements.items():
        text = text.replace(old, new)
    return text


def repair_ocel(text: str) -> str:
    text = text.replace(
        "log.record_op_fired(run_id, 0, 0).unwrap();",
        "log.record_op_fired(run_id, 0, 0, 1).unwrap();",
    )
    text = text.replace(
        "log.record_op_fired(run_id, 1, 0).unwrap();",
        "log.record_op_fired(run_id, 1, 1, 1).unwrap();",
    )
    text = text.replace(
        "log.record_run_sealed(run_id, 0b11).unwrap();",
        "log.record_run_sealed(run_id, 0b11, 2).unwrap();",
    )
    text = text.replace(
        "log.record_run_sealed(81, 1).unwrap();",
        "log.record_run_sealed(81, 1, 0).unwrap();",
        1,
    )
    return text


def repair_chaos_scenarios(text: str) -> str:
    if "fn test_chaos_config_default_is_bounded" not in text:
        marker = "/// Test 1: Crash injection after 5 of 10 sequential operations."
        test = '''#[test]
fn test_chaos_config_default_is_bounded() {
    let config = ChaosConfig::default();
    assert_eq!(config.max_ticks, 100);
    assert_eq!(config.crash_after_tick, None);
    assert_eq!(config.delay_ticks, 0);
    assert!(!config.verify_duplicate_tick_idempotence);
    assert!(!config.reorder_ready_set);
}

'''
        if text.count(marker) != 1:
            raise SystemExit("REPAIR_REFUSED: chaos test insertion marker drift")
        text = text.replace(marker, test + marker, 1)
    needle = "let result = run_with_reorder_injection(&tape, 42, 50);"
    if "reorder injection must execute at least one scheduler tick" not in text:
        replacement = needle + '''

    assert!(
        result.ticks_executed > 0,
        "reorder injection must execute at least one scheduler tick"
    );'''
        if text.count(needle) != 1:
            raise SystemExit("REPAIR_REFUSED: reorder result marker drift")
        text = text.replace(needle, replacement, 1)
    return text


def main() -> int:
    radiation_rel = "crates/bcinr-cmca/tests/usecase_radiation_hardened.rs"
    radiation = edit(radiation_rel, repair_radiation)
    require(radiation_rel, radiation, "results.iter().skip(1)", 1)
    require(radiation_rel, radiation, "repeat.iter().zip(&baseline)", 1)

    fairness_rel = "crates/bcinr-cmca/tests/usecase_ml_fairness_audit.rs"
    fairness = edit(fairness_rel, repair_fairness)
    require(fairness_rel, fairness, "alloc.iter().enumerate().take(N)", 1)
    require(fairness_rel, fairness, "alloc.iter().zip(&baseline).enumerate()", 1)
    require(fairness_rel, fairness, "platform_alloc.iter().zip(&auditor_replay).enumerate()", 1)
    require(fairness_rel, fairness, ".max_by_key(|(_, allocation)| allocation.val)", 1)

    compliance_rel = "crates/bcinr-powl/tests/usecase_compliance_audit.rs"
    compliance = edit(compliance_rel, repair_compliance)
    require(compliance_rel, compliance, "record_op_fired(run_id, op_idx, ticks, 1)", 1)
    if re.search(r"record_op_fired\([^\n]+, 0\)\.unwrap", compliance):
        raise SystemExit(f"REPAIR_REFUSED: {compliance_rel}: stale OCEL call remains")

    chicago_rel = "crates/bcinr-powl/tests/chicago_tdd_integration.rs"
    chicago = edit(chicago_rel, repair_chicago)
    require(chicago_rel, chicago, 'context.insert("start_time"', 1)
    require(chicago_rel, chicago, 'context.insert("duration"', 1)
    require(chicago_rel, chicago, '"event_kind"', 1)
    require(chicago_rel, chicago, "record_op_fired(run_id, op_idx, ticks, 1)", 1)
    require(chicago_rel, chicago, "emit_summary(", 6)
    if "emit_conformance(" in chicago or "event.timestamp" in chicago or "event.kind_tag" in chicago:
        raise SystemExit(f"REPAIR_REFUSED: {chicago_rel}: stale event surface remains")

    ocel_rel = "crates/bcinr-powl/src/ocel.rs"
    ocel = edit(ocel_rel, repair_ocel)
    require(ocel_rel, ocel, "log.record_run_sealed(81, 1, 0)", 1)

    harness_rel = "crates/bcinr-powl/tests/chaos_harness.rs"
    harness = edit(
        harness_rel,
        lambda text: text
        .replace("ticks as u32, 1", "ticks, 1")
        .replace("state.done_mask, ticks as u32", "state.done_mask, ticks"),
    )
    if "ticks as u32" in harness:
        raise SystemExit(f"REPAIR_REFUSED: {harness_rel}: unnecessary cast remains")

    scenarios_rel = "crates/bcinr-powl/tests/chaos_scenarios.rs"
    scenarios = edit(scenarios_rel, repair_chaos_scenarios)
    require(scenarios_rel, scenarios, "fn test_chaos_config_default_is_bounded", 1)
    require(scenarios_rel, scenarios, "result.ticks_executed > 0", 2)

    swarm_rel = "crates/bcinr-powl/tests/usecase_swarm_02_parallel_independent_workers.rs"
    swarm = edit(
        swarm_rel,
        lambda text: text.replace(
            "ticks >= 1 && ticks <= 128,",
            "(1..=128).contains(&ticks),",
            1,
        ),
    )
    require(swarm_rel, swarm, "(1..=128).contains(&ticks)", 1)

    verifier_rel = "scripts/generate_v26_7_26_report.sh"
    verifier = edit(
        verifier_rel,
        lambda text: text
        .replace(
            "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo bench -p bcinr-pddl",
            "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo +nightly-2026-07-24 bench -p bcinr-pddl",
        )
        .replace(
            "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo bench -p bcinr-powl",
            "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo +nightly-2026-07-24 bench -p bcinr-powl",
        ),
    )
    require(verifier_rel, verifier, "cargo +nightly-2026-07-24 bench", 2)

    print("REPAIR_ADMITTED: final source postconditions satisfied")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
