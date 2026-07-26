#!/usr/bin/env python3
"""Fail-closed structural admission checks for BCINR v26.7.26.

This verifier proves source shape only. Executable standing is established by
scripts/generate_v26_7_26_report.sh, which runs the corresponding cargo rails.
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
ERRORS: list[str] = []


def fail(message: str) -> None:
    ERRORS.append(message)


def path(rel: str) -> Path:
    return ROOT / rel


def require_file(rel: str) -> str:
    p = path(rel)
    if not p.is_file():
        fail(f"missing file: {rel}")
        return ""
    return p.read_text(encoding="utf-8")


def require_contains(rel: str, *needles: str) -> str:
    text = require_file(rel)
    for needle in needles:
        if needle not in text:
            fail(f"{rel}: missing {needle!r}")
    return text


def extract_block(text: str, marker: str) -> str:
    start = text.find(marker)
    if start < 0:
        return ""
    brace = text.find("{", start)
    if brace < 0:
        return ""
    depth = 0
    for index in range(brace, len(text)):
        char = text[index]
        if char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
            if depth == 0:
                return text[brace + 1 : index]
    return ""


def rust_code_lines(text: str) -> str:
    """Return executable-looking Rust lines, excluding comment-only evidence.

    The swarm scenarios intentionally document the zero-LLM invariant and show
    forbidden grep examples in doc comments. Those mentions are evidence, not
    calls. Admission therefore scans code and manifests for actuation surfaces
    rather than rejecting negative documentation.
    """
    return "\n".join(
        line for line in text.splitlines() if not line.lstrip().startswith("//")
    )


def phase1() -> None:
    require_file("crates/bcinr-pddl/src/logical_time.rs")
    ground = require_contains(
        "crates/bcinr-pddl/src/ground/mod.rs", "TimeSpecifier", "find_temporal_plan", "deadline"
    )
    if ground.count("TimeSpecifier") < 2:
        fail("crates/bcinr-pddl/src/ground/mod.rs: TimeSpecifier is not threaded beyond one token")
    require_file("crates/bcinr-pddl/tests/temporal_conditions.rs")
    require_file("crates/bcinr-pddl/tests/temporal_deadlines.rs")


def phase2() -> None:
    require_contains(
        "crates/bcinr-pddl/src/resource_ledger.rs",
        "ResourceLease",
        "enum ResourceRefusal",
        "Conflict",
        "enum ResourceMode",
    )
    require_file("crates/bcinr-pddl/tests/resource_leases.rs")


def phase3() -> None:
    scheduler = require_contains(
        "crates/bcinr-powl/src/scheduler.rs",
        "struct PowlRunState",
        "OpTimeInterval",
        "fn intervals_conflict",
        "cancelled_mask",
        "timed_out_mask",
        "refused_mask",
        "blocked_mask",
    )
    body = extract_block(scheduler, "fn intervals_conflict")
    if not body:
        fail("crates/bcinr-powl/src/scheduler.rs: cannot extract intervals_conflict body")
    elif re.search(r"\b(if|match|while|loop)\b", body):
        fail("crates/bcinr-powl/src/scheduler.rs: intervals_conflict contains branch-bearing syntax")
    require_file("crates/bcinr-powl/tests/scheduler_lifecycle.rs")
    require_file("crates/bcinr-powl/tests/scheduler_resource_conflict.rs")


def phase4() -> None:
    ocel = require_contains(
        "crates/bcinr-powl/src/ocel.rs",
        "struct OcelEvent",
        "start_time",
        "duration",
        "DurationViolation",
        "LeaseViolation",
        "DeadlineViolation",
    )
    event_body = extract_block(ocel, "struct OcelEvent")
    for field in ("start_time", "duration"):
        if field not in event_body:
            fail(f"crates/bcinr-powl/src/ocel.rs: OcelEvent missing {field}")
    require_contains("crates/bcinr-powl/src/tape.rs", "deadline", "max_durations")
    require_file("crates/bcinr-powl/tests/ocel_temporal.rs")
    require_file("crates/bcinr-powl/tests/ocel_conformance_temporal.rs")


def phase5() -> None:
    tests = sorted(path("crates/bcinr-powl/tests").glob("usecase_swarm_*.rs"))
    if len(tests) != 10:
        fail(f"expected exactly 10 swarm scenario files, found {len(tests)}")

    # Zero LLM calls means zero executable network/model actuation surfaces.
    # Negative documentation such as "grep openai" is deliberately excluded.
    forbidden_actuation = re.compile(
        r"\b(?:async_openai|anthropic|openai|reqwest|ureq|hyper|"
        r"TcpStream|UdpSocket|std::net|tokio::net|std::process::Command)\b",
        re.IGNORECASE,
    )
    manifest = require_file("crates/bcinr-powl/Cargo.toml")
    if forbidden_actuation.search(rust_code_lines(manifest)):
        fail("crates/bcinr-powl/Cargo.toml: contains an LLM/network actuation dependency")

    for test in tests:
        text = test.read_text(encoding="utf-8")
        if forbidden_actuation.search(rust_code_lines(text)):
            fail(f"{test.relative_to(ROOT)}: contains an executable LLM/network actuation token")
        if not re.search(r"\bassert(?:_eq|_ne)?!", text):
            fail(f"{test.relative_to(ROOT)}: contains no executable assertion")


def phase6() -> None:
    manifest = require_contains(
        "crates/bcinr-ffi/Cargo.toml",
        'version = "26.7.26"',
        'crate-type = ["cdylib", "staticlib", "rlib"]',
    )
    if 'version = "26.7.25"' in manifest:
        fail("crates/bcinr-ffi/Cargo.toml: stale exact 26.7.25 version remains")
    lib = require_contains(
        "crates/bcinr-ffi/src/lib.rs",
        "struct PddlExecutionRequest",
        "struct PowlExecutionRequest",
        'extern "C"',
    )
    if lib.count('extern "C"') < 2:
        fail("crates/bcinr-ffi/src/lib.rs: fewer than two C ABI entry points")
    for struct_name in ("PddlExecutionRequest", "PowlExecutionRequest"):
        block = extract_block(lib, f"struct {struct_name}")
        if "version:" not in block:
            fail(f"crates/bcinr-ffi/src/lib.rs: {struct_name} lacks version field")
    require_file("crates/bcinr-ffi/tests/ffi_conformance.rs")


def phase7() -> None:
    makefile = require_file("Makefile.toml")
    for mutant in range(6, 12):
        if f"mutant_{mutant}" not in makefile:
            fail(f"Makefile.toml: mutant_{mutant} is not wired")
    require_file("crates/bcinr-powl/tests/chaos_harness.rs")
    require_file("crates/bcinr-powl/tests/chaos_scenarios.rs")
    require_file("crates/bcinr-pddl/benches/phase1_temporal.rs")
    require_file("crates/bcinr-powl/benches/phase3_scheduler.rs")
    require_file("scripts/generate_v26_7_26_report.sh")
    require_file(".github/workflows/v26-7-26-release-validation.yml")


PHASES = {
    "1": phase1,
    "2": phase2,
    "3": phase3,
    "4": phase4,
    "5": phase5,
    "6": phase6,
    "7": phase7,
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--phase", choices=[*PHASES, "all"], default="all")
    args = parser.parse_args()

    selected = PHASES.values() if args.phase == "all" else (PHASES[args.phase],)
    for check in selected:
        check()

    if ERRORS:
        for error in ERRORS:
            print(f"STRUCTURE_REFUSED: {error}", file=sys.stderr)
        return 1

    print(f"STRUCTURE_ADMITTED: phase={args.phase}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
