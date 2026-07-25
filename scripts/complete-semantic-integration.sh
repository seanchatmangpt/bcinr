#!/usr/bin/env bash
set -euxo pipefail

MAIN=22945aff08f0d0194febec924c93c5f6a192a942
RECOVERY=8e80292a425207636628c6a489bb9a11c6092208
BASE=3338f59ae5fd11f0f5e05115e2981f6daa8caef2
SCAFFOLD=12419cdac43c953fb1190c5ccbadb4a68e6b6337
BRANCH=agent/semantic-three-way-integration-v26.7.24

git merge-base --is-ancestor "$SCAFFOLD" HEAD
test "$(git merge-base "$MAIN" "$RECOVERY")" = "$BASE"
test "$(git rev-parse "$SCAFFOLD^1")" = "$MAIN"
test "$(git rev-parse "$SCAFFOLD^2")" = "$RECOVERY"

git checkout "$RECOVERY" -- \
  .claude \
  .gitignore \
  CHANGELOG.md \
  Makefile.toml \
  justfile \
  scripts/gates \
  crates/bcinr-cmca \
  crates/bcinr-logic/src/autonomic \
  crates/bcinr-logic/src/patterns/autonomic_arena.rs \
  crates/bcinr-logic/src/patterns/integrity_receipt.rs \
  crates/bcinr-powl/src/auto_select_bridge.rs \
  crates/bcinr-powl/src/auto_select_execution_dispatch.rs \
  crates/bcinr-powl/src/auto_select_final_integration.rs \
  crates/bcinr-powl/src/auto_select_pipeline.rs \
  crates/bcinr-powl/src/auto_select_refusal_aggregation.rs \
  crates/bcinr-powl/src/full_mapek_loop.rs \
  crates/bcinr-powl/src/mapek_loop.rs \
  crates/bcinr-powl/src/scheduler.rs \
  crates/bcinr-powl-receipt/src/causal_buffer_integration.rs \
  crates/bcinr-powl-receipt/tests/hostile_mutants.rs \
  crates/bcinr-powl-receipt/README.md \
  tools/bcinr-cmca-audit-harness \
  docs/architecture/v26.7.18 \
  docs/product/v26.7.18 \
  docs/jira/v26.7.18 \
  docs/cmca-rdf \
  docs/constitution-compiler

# Main owns production-generated sources, compile-fail harness shape, and existing primitives.
git checkout "$MAIN" -- \
  crates/bcinr-cmca/src/generated \
  crates/bcinr-cmca/tests/compile_fail_tests.rs \
  crates/bcinr-logic/src/autonomic/autonomic_substrate.rs \
  crates/bcinr-logic/src/autonomic/kernel.rs \
  crates/bcinr-logic/src/autonomic/metric_accumulator.rs \
  crates/bcinr-logic/src/autonomic/policy_guard.rs \
  crates/bcinr-logic/src/autonomic/rl_state.rs

python3 <<'PY'
from pathlib import Path
import subprocess

root = Path('.')

cargo = root / 'Cargo.toml'
text = cargo.read_text()
text = text.replace(
    '"crates/bcinr-mcp", "crates/bcinr-mfw-ir"]',
    '"crates/bcinr-mcp", "crates/bcinr-mfw-ir", "tools/bcinr-cmca-audit-harness"]',
)
text = text.replace(
    'over the *entire* dependency graph (nom, the pddl parser,\n# wasm4pm-compat, criterion, ...)',
    'over the *entire* dependency graph (wasm4pm-compat, criterion, ...)',
)
cargo.write_text(text)

(root / 'crates/bcinr-cmca/Cargo.toml').write_text('''[package]
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
''')

cmca_lib = root / 'crates/bcinr-cmca/src/lib.rs'
text = cmca_lib.read_text()
text = text.replace(
    '#[cfg(test)]\npub mod artifact;',
    '#[cfg(any(test, feature = "artifact-verification"))]\npub mod artifact;',
)
text = text.replace(
    '// Gated to #[cfg(test)] because its dependencies (blake3, serde, serde_json)\n'
    '// are dev-dependencies only, keeping non-test builds of this crate free of\n'
    '// any additional runtime dependency beyond bcinr-logic. See src/artifact.rs\n'
    '// module docs for the full rationale.',
    '// Available to tests and through the explicit `artifact-verification` feature.\n'
    '// The production hot path remains free of these slow-rail dependencies by default.',
)
cmca_lib.write_text(text)

autonomic = root / 'crates/bcinr-logic/src/autonomic/mod.rs'
text = autonomic.read_text()
text = text.replace('pub mod kernel;', '#[cfg(feature = "alloc")]\npub mod kernel;')
text = text.replace('pub use kernel::{\n', '#[cfg(feature = "alloc")]\npub use kernel::{\n')
autonomic.write_text(text)

# AutoSelect needs two width-complete primitives absent from main's mask surface.
# Add only those lawful branchless operations; do not replace main's mask module.
mask = root / 'crates/bcinr-logic/src/mask.rs'
text = mask.read_text()
if 'pub const fn select_u8(' not in text:
    text += '''

/// Branchless conditional select for an all-ones/all-zeros `u8` mask.
#[inline(always)]
#[must_use = "branchless select — ignoring this result discards the computed selection"]
pub const fn select_u8(mask: u8, a: u8, b: u8) -> u8 {
    (mask & a) | (!mask & b)
}
'''
if 'pub const fn is_zero_mask_u64(' not in text:
    text += '''

/// Branchless zero-test mask for `u64` values.
#[inline(always)]
#[must_use = "branchless zero mask — ignoring this result discards the zero-test"]
pub const fn is_zero_mask_u64(x: u64) -> u64 {
    let non_zero_msb = (x | x.wrapping_neg()) >> 63;
    non_zero_msb.wrapping_sub(1)
}
'''
mask.write_text(text)

powl = root / 'crates/bcinr-powl/src/lib.rs'
text = powl.read_text()
additions = (
    'pub mod auto_select_bridge;\n'
    'pub mod auto_select_execution_dispatch;\n'
    'pub mod auto_select_final_integration;\n'
    'pub mod auto_select_pipeline;\n'
    'pub mod auto_select_refusal_aggregation;\n'
)
if 'pub mod auto_select_bridge;' not in text:
    text = text.replace('pub mod admit;\n', 'pub mod admit;\n' + additions)
if 'pub mod full_mapek_loop;' not in text:
    text = text.replace('pub mod enterprise;\n', 'pub mod enterprise;\npub mod full_mapek_loop;\npub mod mapek_loop;\n')
powl.write_text(text)

receipt = root / 'crates/bcinr-powl-receipt/src/lib.rs'
text = receipt.read_text()
if 'pub mod causal_buffer_integration;' not in text:
    text = text.replace('pub mod causal_receipt;\n', 'pub mod causal_buffer_integration;\npub mod causal_receipt;\n')
receipt.write_text(text)

main_agents = subprocess.check_output(['git', 'show', '22945aff08f0d0194febec924c93c5f6a192a942:AGENTS.md'], text=True)
recovery_agents = subprocess.check_output(['git', 'show', '8e80292a425207636628c6a489bb9a11c6092208:AGENTS.md'], text=True)
marker = '# Appendix: Claude Code operating model'
appendix = recovery_agents[recovery_agents.index(marker):]
(root / 'AGENTS.md').write_text(main_agents.rstrip() + '\n\n---\n\n' + appendix.rstrip() + '\n')
PY

rm -f .github/workflows/pr11-merge-main.yml
rm -f ./*.rlib ./audit_results*.log ./auditor_output.txt ./maturity_results*.txt
rm -f ./test_output.log ./test-mutants-output.log ./scratch.py ./scratch.rs
rm -f ./test_derive ./test_lt ./test_mutant10
find . -maxdepth 1 -type f \( \
  -name 'fix_*.py' -o -name 'patch*.py' -o -name 'patch*.diff' -o \
  -name 'wipe_bridges.py' -o -name 'resolve_*.py' \
\) -delete

find .claude crates/bcinr-cmca docs/architecture/v26.7.18 \
  docs/product/v26.7.18 docs/jira/v26.7.18 docs/cmca-rdf \
  docs/constitution-compiler tools/bcinr-cmca-audit-harness \
  -type f \( -name '*.rs' -o -name '*.toml' -o -name '*.md' -o \
  -name '*.json' -o -name '*.ttl' -o -name '*.yaml' -o -name '*.yml' \) \
  -exec chmod 644 {} +

python3 - <<'PY'
import tomllib
from pathlib import Path
for path in (
    Path('Cargo.toml'),
    Path('crates/bcinr-cmca/Cargo.toml'),
    Path('crates/bcinr-powl/Cargo.toml'),
    Path('crates/bcinr-powl-receipt/Cargo.toml'),
):
    tomllib.loads(path.read_text())
PY

cargo generate-lockfile
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace --all-features 2>&1 | tee integration-check.log
{
  cargo test -p bcinr-cmca --all-features --no-fail-fast
  cargo test -p bcinr-cmca --test calibration
  cargo test -p bcinr-cmca --test differential
  cargo test -p bcinr-cmca --test hostile_mutants
  cargo test -p bcinr-pddl --all-features
  cargo test -p bcinr-powl --all-features
  cargo test -p bcinr-powl --all-features powl2
  cargo test -p bcinr-powl --all-features scheduler_v2
  cargo test -p bcinr-powl-receipt --all-features
  cargo test -p bcinr-powl-receipt --all-features execution_v2
  cargo check -p bcinr-cmca-audit-harness
} 2>&1 | tee integration-tests.log

rm -f .github/workflows/integration-materialize.yml
rm -f .github/workflows/integration-execute.yml
rm -f scripts/complete-semantic-integration.sh

git config user.name 'OpenAI Integration Agent'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git add -A
git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py'
git commit -m 'feat: complete semantic recovery integration'
git push origin HEAD:"$BRANCH"
