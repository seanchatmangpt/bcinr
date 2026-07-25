#!/usr/bin/env python3
"""Normalize recovered test fixtures without weakening production lint policy."""

from pathlib import Path


def add_test_lint_boundary(path: Path, lints: tuple[str, ...], rationale: str) -> None:
    source = path.read_text()
    marker = "#[cfg(test)]\nmod tests {"
    if marker not in source:
        raise RuntimeError(f"{path}: test module marker missing")
    lint_list = ", ".join(f"clippy::{lint}" for lint in lints)
    replacement = (
        "#[cfg(test)]\n"
        f"// {rationale}\n"
        f"#[allow({lint_list})]\n"
        "mod tests {"
    )
    path.write_text(source.replace(marker, replacement, 1))


# The adapter adds two production helpers needed by recovered modules. Keep them
# before the test module so all-target Clippy sees canonical item ordering.
mask = Path("crates/bcinr-logic/src/mask.rs")
source = mask.read_text()
helper_marker = "\n\n/// Branchless conditional select for an all-ones/all-zeros `u8` mask."
test_marker = "\n#[cfg(test)]\nmod tests {"
helper_start = source.find(helper_marker)
if helper_start < 0:
    raise RuntimeError("mask compatibility helpers missing")
if test_marker not in source:
    raise RuntimeError("mask test module marker missing")
helpers = source[helper_start:].rstrip()
source = source[:helper_start].rstrip() + "\n"
source = source.replace(test_marker, "\n" + helpers + test_marker, 1)
mask.write_text(source)

# Recovery tests intentionally construct state incrementally to mirror the
# proposal/admission/refusal sequence and fork Copy snapshots for hostile-mutant
# comparisons. These narrow test-only boundaries retain that audit shape while
# production and non-test code remain under workspace -D warnings.
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/auto_select.rs"),
    ("collapsible_if", "field_reassign_with_default"),
    "The oracle mirrors the staged admission calculus and test fixtures mutate one gate at a time.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/auto_select_epoch_reclamation.rs"),
    ("field_reassign_with_default",),
    "Epoch fixtures expose each state transition explicitly for hostile-mutant comparison.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/auto_select_execution_dispatch.rs"),
    ("field_reassign_with_default",),
    "Dispatch fixtures stage admitted and refused results in the same mutable carrier.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/auto_select_ocel_emission.rs"),
    ("field_reassign_with_default", "clone_on_copy"),
    "OCEL tests fork bounded buffer snapshots and mutate timestamps to prove causal refusal invariance.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/auto_select_trace_logging.rs"),
    ("field_reassign_with_default", "clone_on_copy"),
    "Trace tests fork bounded snapshots and mutate envelope fields to isolate hostile paths.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/canonical_mass.rs"),
    ("field_reassign_with_default", "clone_on_copy"),
    "Canonical-mass tests fork admitted inputs and alter one refusal dimension per mutant.",
)
add_test_lint_boundary(
    Path("crates/bcinr-logic/src/autonomic/receipt_integration.rs"),
    ("field_reassign_with_default",),
    "Receipt fixtures stage prior and candidate weights explicitly to audit masked commit behavior.",
)

# This is a one-run migration adapter. Remove it from the materialized audited
# tree after its deterministic transformations have been applied.
Path(__file__).unlink()
