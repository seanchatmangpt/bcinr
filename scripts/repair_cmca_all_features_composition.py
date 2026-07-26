#!/usr/bin/env python3
"""Fence multi-mutant composition and remove the legacy LSP fallback."""

from pathlib import Path
import re

path = Path("crates/bcinr-cmca/tests/hostile_mutants.rs")
source = path.read_text()

anchor = '\n#[cfg(feature = "mutant_1")]\n'
helper = '''
fn require_uncomposed_mutant(test: &str) -> bool {
    if ACTIVE_MUTANT_COUNT <= 1 {
        true
    } else {
        eprintln!(
            "BCINR_TYPED_SKIP[cmca-mutant-composition]: {test} requires zero or one active mutant feature; observed {ACTIVE_MUTANT_COUNT}"
        );
        false
    }
}

#[cfg(feature = "mutant_1")]
'''
if source.count(anchor) != 1:
    raise RuntimeError("isolated mutant section anchor missing")
source = source.replace(anchor, helper, 1)

for test in (
    "kill_m01_ignore_numeric_error",
    "kill_m03_point_estimate_gram_gate",
    "kill_m05_ignore_drift",
    "kill_m07_ignore_gram",
):
    pattern = re.compile(rf"(fn {test}\(\) \{{\n)")
    guard = (
        rf'\1    if !require_uncomposed_mutant("{test}") {{\n'
        "        return;\n"
        "    }\n"
    )
    source, count = pattern.subn(guard, source, count=1)
    if count != 1:
        raise RuntimeError(f"{test}: composition guard insertion failed")

path.write_text(source)

e2e = Path("bcinr/tests/e2e_main.rs")
source = e2e.read_text()
legacy = '''pub fn run_lsp_cmd(dir: &str) -> std::process::Output {
    ensure_lsp_built();
    let target_dir = std::env::temp_dir().join("bcinr-e2e-target");
    let lsp_binary = target_dir.join("debug/anti-llm-cheat-lsp");
    if !lsp_binary.exists() {
        eprintln!(
            "anti-llm-cheat-lsp binary not found at {:?}, returning empty output",
            lsp_binary
        );
        // Return a dummy output that indicates the test should be skipped
        return std::process::Command::new("true").output().unwrap();
    }
    let mut cmd = Command::new(&lsp_binary);
    cmd.arg("scan");
    cmd.args(["--dir", dir]);
    cmd.current_dir(get_repo_root());
    cmd.output().unwrap()
}

'''
if source.count(legacy) != 1:
    raise RuntimeError("legacy dummy-success run_lsp_cmd implementation missing")
e2e.write_text(source.replace(legacy, "", 1))
