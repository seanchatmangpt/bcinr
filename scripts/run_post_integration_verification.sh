#!/usr/bin/env bash
set -euo pipefail

BRANCH=agent/repair-post-integration-verification-ci
RECEIPT=docs/integration/post-pr14-verification-repair-v26.7.25.md

mkdir -p "$(dirname "$RECEIPT")"
cat > "$RECEIPT" <<'EOF'
# Post-PR #14 verification and CI repair receipt

## Ground truth

- Base commit: `9ccc5ec308fce6bab6f2d81cdd7034eea77db64c`
- Former PR #14 head: `66a65853d5fc65d50cab7ffa92febabcdd5a477b`
- Initial broad E2E standing: `42 passed; 18 failed`
- Initial classification: `PARTIAL_ALIVE / BUILD_BROKEN`

## Repairs

- Added the missing `u64_contract!` marker to `select_u32`.
- Made hostile-mutant assertions require exactly one active mutant feature while preserving all eleven isolated rails.
- Replaced machine-local anti-LLM LSP assumptions with explicit typed skips when the sibling repository is not admitted.
- Made formatting fixtures reachable by `cargo fmt` and converted the nonexistent-directory case into a typed failure assertion.
- Split permanent CI into command-level receipts and converted exhaustive mutants into an eleven-job matrix.
- Removed post-PR13 executors, diagnostics, publishers, adapters, and integration-only scripts.

## Verification

| Command | Exit | Elapsed seconds | Classification |
|---|---:|---:|---|
EOF

run_required() {
  local label="$1"
  shift
  local log
  local start
  local end
  local elapsed
  local status
  local classification
  log="$(mktemp)"
  start="$(date +%s)"
  if "$@" >"$log" 2>&1; then
    status=0
    classification=ALIVE
  else
    status=$?
    classification=BUILD_BROKEN
  fi
  end="$(date +%s)"
  elapsed=$((end - start))
  cat "$log"
  rm -f "$log"
  printf '| `%s` | %s | %s | %s |\n' "$label" "$status" "$elapsed" "$classification" >> "$RECEIPT"
  if [[ "$status" -ne 0 ]]; then
    printf '\n## Final classification\n\n`BUILD_BROKEN` at `%s`.\n' "$label" >> "$RECEIPT"
    exit "$status"
  fi
}

run_required 'cargo fmt --all -- --check' cargo fmt --all -- --check
run_required 'cargo check --workspace --all-features' cargo check --workspace --all-features
run_required 'cargo clippy --workspace --all-targets --all-features -- -D warnings' cargo clippy --workspace --all-targets --all-features -- -D warnings
run_required 'cargo test -p bcinr-cmca --all-features' cargo test -p bcinr-cmca --all-features
run_required 'cargo test -p bcinr-cmca --test calibration' cargo test -p bcinr-cmca --test calibration
run_required 'cargo test -p bcinr-cmca --test differential' cargo test -p bcinr-cmca --test differential
run_required 'cargo test -p bcinr-cmca --test hostile_mutants' cargo test -p bcinr-cmca --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --test compile_fail_tests' cargo test -p bcinr-cmca --test compile_fail_tests
run_required 'cargo test -p bcinr-cmca --features mutant_1 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_1 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_2 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_2 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_3 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_3 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_4 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_4 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_5 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_5 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_6 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_6 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_7 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_7 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_8 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_8 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_9 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_9 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_10 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_10 --test hostile_mutants
run_required 'cargo test -p bcinr-cmca --features mutant_11 --test hostile_mutants' cargo test -p bcinr-cmca --features mutant_11 --test hostile_mutants
run_required 'cargo test -p bcinr-pddl --all-features' cargo test -p bcinr-pddl --all-features
run_required 'cargo test -p bcinr-powl --all-features' cargo test -p bcinr-powl --all-features
run_required 'cargo test -p bcinr-powl --all-features powl2' cargo test -p bcinr-powl --all-features powl2
run_required 'cargo test -p bcinr-powl --all-features scheduler_v2' cargo test -p bcinr-powl --all-features scheduler_v2
run_required 'cargo test -p bcinr-powl-receipt --all-features' cargo test -p bcinr-powl-receipt --all-features
run_required 'cargo test -p bcinr-powl-receipt --all-features execution_v2' cargo test -p bcinr-powl-receipt --all-features execution_v2
run_required 'cargo test -p bcinr --test e2e_main' cargo test -p bcinr --test e2e_main
run_required 'cargo test --workspace --all-features --all-targets --no-fail-fast' cargo test --workspace --all-features --all-targets --no-fail-fast
run_required 'post-E2E cargo fmt --all -- --check' cargo fmt --all -- --check
run_required 'post-E2E algorithms/mod.rs remains unchanged' git diff --exit-code -- crates/bcinr-logic/src/algorithms/mod.rs

git diff --check
if find crates/bcinr-logic/src/algorithms -maxdepth 1 -type f -name 'temp_*' -print -quit | grep -q .; then
  echo 'E2E residue remained in crates/bcinr-logic/src/algorithms' >&2
  exit 1
fi

cat >> "$RECEIPT" <<'EOF'

## Final classification

`ALIVE` for the observed Linux verification ladder. Automatic PR CI may now be admitted.
EOF

rm -f .github/workflows/repair-post-integration-verification-v2.yml
rm -f scripts/repair_post_integration_verification.py
rm -f scripts/run_post_integration_verification.sh

test ! -e .github/workflows/post-pr13-audit.yml
test ! -e .github/workflows/post-pr13-pr-audit.yml
test ! -e scripts/adapt_semantic_integration.py
test ! -e scripts/complete_semantic_integration.py
test ! -e scripts/integration_adapt_current_apis.py
test ! -e scripts/normalize_recovered_tests.py
test ! -e scripts/publish-semantic-integration-v5.sh
test ! -e scripts/run-semantic-integration-v4.sh

git config user.name 'OpenAI Repair Agent'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git add -A
git diff --cached --check
git commit -m 'fix: close post-integration verification and CI defects'
git push origin "HEAD:${BRANCH}"
