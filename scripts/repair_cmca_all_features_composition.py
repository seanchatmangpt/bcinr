#!/usr/bin/env python3
"""Fence multi-mutant feature composition without weakening isolated mutant rails."""

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
