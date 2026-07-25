#!/usr/bin/env python3
"""Complete PR #13 from the exact base/main/recovery object graph."""

from __future__ import annotations

import io
import os
import shutil
import subprocess
import tarfile
from pathlib import Path

ROOT = Path.cwd()
TMP = ROOT / ".integration-trees"
BASE_SHA = "3338f59ae5fd11f0f5e05115e2981f6daa8caef2"
MAIN_SHA = "22945aff08f0d0194febec924c93c5f6a192a942"
RECOVERY_SHA = "8e80292a425207636628c6a489bb9a11c6092208"


def extract_tree(sha: str, destination: Path) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    archive = subprocess.check_output(["git", "archive", sha])
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tf:
        tf.extractall(destination)


def resolve_diff3(text: str, prefer: str) -> str:
    lines = text.splitlines(keepends=True)
    result: list[str] = []
    i = 0
    while i < len(lines):
        if not lines[i].startswith("<<<<<<<"):
            result.append(lines[i])
            i += 1
            continue
        i += 1
        main_block: list[str] = []
        while i < len(lines) and not lines[i].startswith("|||||||"):
            main_block.append(lines[i])
            i += 1
        if i == len(lines):
            raise RuntimeError("malformed diff3 conflict: missing base marker")
        i += 1
        while i < len(lines) and not lines[i].startswith("======="):
            i += 1
        if i == len(lines):
            raise RuntimeError("malformed diff3 conflict: missing separator")
        i += 1
        recovery_block: list[str] = []
        while i < len(lines) and not lines[i].startswith(">>>>>>>"):
            recovery_block.append(lines[i])
            i += 1
        if i == len(lines):
            raise RuntimeError("malformed diff3 conflict: missing end marker")
        i += 1
        result.extend(recovery_block if prefer == "recovery" else main_block)
    return "".join(result)


def merge_file(rel: str, prefer: str) -> None:
    relp = Path(rel)
    main_path = MAIN / relp
    base_path = BASE / relp
    recovery_path = RECOVERY / relp
    if not (main_path.exists() and base_path.exists() and recovery_path.exists()):
        raise FileNotFoundError(f"three-way input missing for {rel}")
    proc = subprocess.run(
        [
            "git",
            "merge-file",
            "-p",
            "--diff3",
            str(main_path),
            str(base_path),
            str(recovery_path),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    target = ROOT / relp
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(resolve_diff3(proc.stdout, prefer))


def copy_recovery_only(rel_root: str) -> None:
    source_root = RECOVERY / rel_root
    if not source_root.exists():
        return
    for source in source_root.rglob("*"):
        if not source.is_file():
            continue
        rel = source.relative_to(RECOVERY)
        if (MAIN / rel).exists():
            continue
        target = ROOT / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, target)


def write_cmca_manifest() -> None:
    (ROOT / "crates/bcinr-cmca/Cargo.toml").write_text(
        """[package]
name = "bcinr-cmca"
version = "26.6.24"
edition = "2021"
description = "CMCA-RDF deterministic substrate crate"
license = "MIT OR Apache-2.0"
rust-version = "1.70"
repository = "https://github.com/seanchatmangpt/bcinr"
keywords = ["cmca", "rdf", "deterministic", "algorithms"]
categories = ["algorithms"]
readme = "../../README.md"
exclude = ["quarantine/**"]

[dependencies]
bcinr-logic = { path = "../bcinr-logic", version = "26.6.24" }
blake3 = { version = "1", optional = true }
serde = { version = "1", features = ["derive"], optional = true }
serde_json = { version = "1", optional = true }

[dev-dependencies]
trybuild = "1.0"
proptest = "1.2.0"
blake3 = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chicago-tdd-tools = { version = "26.7.1", features = ["ocel-generation"] }

[features]
default = []
alloc = []
std = ["bcinr-logic/std", "alloc"]
artifact-verification = ["std", "dep:blake3", "dep:serde", "dep:serde_json"]
alloc-gate = []
mutant_1 = []
mutant_2 = []
mutant_3 = []
mutant_4 = []
mutant_5 = []
mutant_6 = []
mutant_7 = []
mutant_8 = []
mutant_9 = []
mutant_10 = []
mutant_11 = []

[lib]
crate-type = ["rlib"]
"""
    )


def write_execution_v2_hostile_tests() -> None:
    path = ROOT / "crates/bcinr-powl-receipt/tests/hostile_mutants.rs"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(
        r'''use bcinr_powl::powl2::{compile_powl2, LowestIndexPolicy, Powl2Model};
use bcinr_powl::tape::v2::ConcurrencyGuardTable;
use bcinr_powl_receipt::execution_v2::{
    execute_and_seal_v2, verify_execution_v2, PowlV2ReceiptError,
};

fn compiled() -> bcinr_powl::powl2::CompiledPowl2 {
    compile_powl2(
        &Powl2Model::Sequence(vec![
            Powl2Model::Activity("observe".into()),
            Powl2Model::Activity("act".into()),
        ]),
        &mut LowestIndexPolicy,
    )
    .unwrap()
}

#[test]
fn forged_tape_root_is_refused() {
    let compiled = compiled();
    let guards = ConcurrencyGuardTable::empty();
    let mut receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
    receipt.tape_root.push('0');
    assert_eq!(
        verify_execution_v2(&receipt, &compiled.tape, &guards, 8),
        Err(PowlV2ReceiptError::TapeRootMismatch)
    );
}

#[test]
fn forged_firing_trace_is_refused() {
    let compiled = compiled();
    let guards = ConcurrencyGuardTable::empty();
    let mut receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
    receipt.fired_masks[0] ^= 1;
    assert_eq!(
        verify_execution_v2(&receipt, &compiled.tape, &guards, 8),
        Err(PowlV2ReceiptError::FiredTraceMismatch)
    );
}

#[test]
fn forged_final_state_is_refused() {
    let compiled = compiled();
    let guards = ConcurrencyGuardTable::empty();
    let mut receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
    receipt.final_done_mask ^= 1;
    assert_eq!(
        verify_execution_v2(&receipt, &compiled.tape, &guards, 8),
        Err(PowlV2ReceiptError::FinalStateMismatch)
    );
}

#[test]
fn forged_chain_root_is_refused() {
    let compiled = compiled();
    let guards = ConcurrencyGuardTable::empty();
    let mut receipt = execute_and_seal_v2(&compiled.tape, &guards, 8).unwrap();
    receipt.chain_root.push('0');
    assert_eq!(
        verify_execution_v2(&receipt, &compiled.tape, &guards, 8),
        Err(PowlV2ReceiptError::ChainRootMismatch)
    );
}
'''
    )


def cleanup() -> None:
    patterns = [
        "*.rlib",
        "audit_results*",
        "auditor_output.txt",
        "maturity_results*.txt",
        "test_output.log",
        "test-mutants-output.log",
        "scratch.py",
        "scratch.rs",
        "fix_*.py",
        "patch*.py",
        "patch*.diff",
        "wipe_bridges.py",
        "resolve_*.py",
    ]
    for pattern in patterns:
        for path in ROOT.glob(pattern):
            if path.is_dir():
                shutil.rmtree(path)
            else:
                path.unlink(missing_ok=True)
    for name in ["test_derive", "test_lt", "test_mutant10", "libscratch2.rlib"]:
        path = ROOT / name
        if path.exists() or path.is_symlink():
            path.unlink()
    for rel in [
        ".github/workflows/pr11-merge-main.yml",
        ".github/workflows/integration-materialize.yml",
        "scripts/complete_semantic_integration.py",
    ]:
        (ROOT / rel).unlink(missing_ok=True)


def normalize_modes() -> None:
    text_suffixes = {".rs", ".toml", ".md", ".json", ".ttl", ".yaml", ".yml"}
    roots = [
        ROOT / ".claude",
        ROOT / "crates/bcinr-cmca",
        ROOT / "crates/bcinr-logic/src/autonomic",
        ROOT / "crates/bcinr-powl",
        ROOT / "crates/bcinr-powl-receipt",
        ROOT / "tools/bcinr-cmca-audit-harness",
        ROOT / "docs",
    ]
    for base_dir in roots:
        if not base_dir.exists():
            continue
        for path in base_dir.rglob("*"):
            if path.is_file() and path.suffix in text_suffixes:
                os.chmod(path, 0o644)
    gates = ROOT / "scripts/gates"
    if gates.exists():
        for path in gates.rglob("*.sh"):
            os.chmod(path, 0o755)


def write_report() -> None:
    report = ROOT / "docs/integration/semantic-three-way-merge-v26.7.24.md"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(
        """# Semantic three-way integration — v26.7.24

## Merge basis

- Common ancestor: `3338f59ae5fd11f0f5e05115e2981f6daa8caef2`
- Production main: `22945aff08f0d0194febec924c93c5f6a192a942`
- Recovery source: `8e80292a425207636628c6a489bb9a11c6092208`
- Two-parent scaffold: `12419cdac43c953fb1190c5ccbadb4a68e6b6337`

## Integrated result

Production main remains authoritative for workspace dependency direction, CI and Miri, PDDL v2, POWL v2, `scheduler_v2`, receipt `execution_v2`, algorithm source, and file modes. Recovery contributes CMCA typed-refusal and authority-separation modules, artifact verification, shadow execution, jump and stability analysis, certified mode switching, AutoSelect and MAPE-K layers, causal-buffer integration, hostile verification, governance, audit tooling, and release documentation.

Shared CMCA runtime files were produced from a real diff3 merge over the stated ancestor. Non-overlapping changes from both sides were retained; recovery semantics were selected only inside genuine CMCA overlap regions because those regions define the recovered authority chain. Shared POWL and receipt files were also diff3 merged, with main selected in genuine conflicts so POWL v2 and execution-v2 semantics remain foundational.

## Generated artifacts

`crates/bcinr-cmca/src/generated/*.rs` are retained byte-for-byte from production main. They were not line-merged. Recovery's legacy generator and ontology remain quarantined under `crates/bcinr-cmca/quarantine/`; recovery producer artifacts and manifests remain under `generated-artifact/` for verification. The current repository contains no authoritative post-split generator capable of lawfully reproducing the production generated files, so no output was fabricated.

## Cleanup and modes

Recovery logs, `.rlib` files, scratch files, temporary binaries, one-off repair scripts, temporary patches, and the PR-11 merge workflow are removed. Existing algorithm and playground source remain from main, so mode-only recovery noise is absent. Imported Rust, TOML, Markdown, JSON, RDF, and YAML files are normalized to mode `100644`; supported shell gates remain executable.

## Validation

The resulting commit regenerates `Cargo.lock` and is admitted only after the complete validation ladder in PR #13's `Complete Semantic Integration V2` run succeeds. GitHub Actions logs and artifacts are the executable receipt.
"""
    )


if TMP.exists():
    shutil.rmtree(TMP)
extract_tree(BASE_SHA, TMP / "base")
extract_tree(MAIN_SHA, TMP / "main")
extract_tree(RECOVERY_SHA, TMP / "recovery")
BASE = TMP / "base"
MAIN = TMP / "main"
RECOVERY = TMP / "recovery"

for rel in ["Cargo.toml", ".gitignore", "Makefile.toml"]:
    merge_file(rel, "recovery")
for rel in ["CHANGELOG.md", "justfile"]:
    if (BASE / rel).exists() and (MAIN / rel).exists() and (RECOVERY / rel).exists():
        merge_file(rel, "recovery")

main_agents = (MAIN / "AGENTS.md").read_text()
recovery_agents = (RECOVERY / "AGENTS.md").read_text()
marker = "# Appendix: Claude Code operating model"
(ROOT / "AGENTS.md").write_text(
    main_agents.rstrip() + "\n\n---\n\n" + recovery_agents[recovery_agents.index(marker) :].rstrip() + "\n"
)

merge_file("crates/bcinr-logic/src/autonomic/mod.rs", "recovery")
autonomic_mod = ROOT / "crates/bcinr-logic/src/autonomic/mod.rs"
autonomic_text = autonomic_mod.read_text()
autonomic_text = autonomic_text.replace(
    "pub mod kernel;", '#[cfg(feature = "alloc")]\npub mod kernel;'
)
autonomic_text = autonomic_text.replace(
    "pub use kernel::{\n",
    '#[cfg(feature = "alloc")]\npub use kernel::{\n',
)
autonomic_mod.write_text(autonomic_text)

for rel in [
    "crates/bcinr-cmca/src/allocator.rs",
    "crates/bcinr-cmca/src/fixed.rs",
    "crates/bcinr-cmca/src/lib.rs",
    "crates/bcinr-cmca/src/lrc.rs",
    "crates/bcinr-cmca/src/observatory.rs",
    "crates/bcinr-cmca/tests/calibration.rs",
    "crates/bcinr-cmca/tests/case_studies.rs",
    "crates/bcinr-cmca/tests/differential.rs",
    "crates/bcinr-cmca/tests/hostile_mutants.rs",
    "crates/bcinr-cmca/tests/reference.rs",
    "crates/bcinr-cmca/tests/compile_fail_tests.rs",
]:
    merge_file(rel, "recovery")

for rel in [
    "crates/bcinr-powl/src/admit.rs",
    "crates/bcinr-powl/src/compiler.rs",
    "crates/bcinr-powl/src/dispatcher.rs",
    "crates/bcinr-powl/src/enterprise.rs",
    "crates/bcinr-powl/src/lib.rs",
    "crates/bcinr-powl/src/ocel.rs",
    "crates/bcinr-powl/src/projection.rs",
    "crates/bcinr-powl/src/scheduler.rs",
    "crates/bcinr-powl/src/typestate.rs",
    "crates/bcinr-powl-receipt/src/lib.rs",
    "crates/bcinr-powl-receipt/src/replay.rs",
]:
    merge_file(rel, "main")
shutil.copy2(MAIN / "crates/bcinr-powl/Cargo.toml", ROOT / "crates/bcinr-powl/Cargo.toml")
shutil.copy2(
    MAIN / "crates/bcinr-powl-receipt/Cargo.toml",
    ROOT / "crates/bcinr-powl-receipt/Cargo.toml",
)

for rel_root in [
    ".claude",
    "crates/bcinr-cmca",
    "crates/bcinr-logic/src/autonomic",
    "crates/bcinr-logic/src/patterns",
    "crates/bcinr-powl",
    "crates/bcinr-powl-receipt",
    "tools/bcinr-cmca-audit-harness",
    "scripts/gates",
    "docs/architecture/v26.7.18",
    "docs/product/v26.7.18",
    "docs/jira/v26.7.18",
    "docs/cmca-rdf",
    "docs/constitution-compiler",
]:
    copy_recovery_only(rel_root)

for rel in [
    "crates/bcinr-cmca/src/generated/case_studies.rs",
    "crates/bcinr-cmca/src/generated/generalization.rs",
    "crates/bcinr-cmca/src/generated/stability_profile.rs",
]:
    shutil.copy2(MAIN / rel, ROOT / rel)

write_cmca_manifest()
cmca_lib = ROOT / "crates/bcinr-cmca/src/lib.rs"
cmca_text = cmca_lib.read_text()
cmca_text = cmca_text.replace(
    "#[cfg(test)]\npub mod artifact;",
    '#[cfg(any(test, feature = "artifact-verification"))]\npub mod artifact;',
)
cmca_lib.write_text(cmca_text)

receipt_lib = ROOT / "crates/bcinr-powl-receipt/src/lib.rs"
receipt_text = receipt_lib.read_text()
if "pub mod causal_buffer_integration;" not in receipt_text:
    receipt_text = receipt_text.replace(
        "pub mod causal_receipt;",
        "pub mod causal_buffer_integration;\npub mod causal_receipt;",
    )
receipt_lib.write_text(receipt_text)
write_execution_v2_hostile_tests()

cargo = ROOT / "Cargo.toml"
cargo_text = cargo.read_text()
if '"tools/bcinr-cmca-audit-harness"' not in cargo_text:
    cargo_text = cargo_text.replace(
        '"crates/bcinr-mcp", "crates/bcinr-mfw-ir"]',
        '"crates/bcinr-mcp", "crates/bcinr-mfw-ir", "tools/bcinr-cmca-audit-harness"]',
    )
cargo_text = cargo_text.replace(
    "(nom, the pddl parser,\n# wasm4pm-compat, criterion, ...)",
    "(wasm4pm-compat, criterion, ...)",
)
cargo.write_text(cargo_text)

write_report()
normalize_modes()
cleanup()
shutil.rmtree(TMP, ignore_errors=True)
