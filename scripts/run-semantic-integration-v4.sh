#!/usr/bin/env bash
set -euxo pipefail

MAIN=22945aff08f0d0194febec924c93c5f6a192a942
RECOVERY=8e80292a425207636628c6a489bb9a11c6092208
BASE=3338f59ae5fd11f0f5e05115e2981f6daa8caef2
SCAFFOLD=12419cdac43c953fb1190c5ccbadb4a68e6b6337
BRANCH=agent/semantic-three-way-integration-v26.7.24

git merge-base --is-ancestor "$SCAFFOLD" HEAD
test "$(git rev-parse "$SCAFFOLD^1")" = "$MAIN"
test "$(git rev-parse "$SCAFFOLD^2")" = "$RECOVERY"
test "$(git merge-base "$MAIN" "$RECOVERY")" = "$BASE"

python3 scripts/complete_semantic_integration.py
python3 scripts/adapt_semantic_integration.py

cargo generate-lockfile
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace --all-features 2>&1 | tee integration-check.log
{
  cargo test -p bcinr-cmca --all-features --no-fail-fast
  cargo test -p bcinr-cmca --test calibration
  cargo test -p bcinr-cmca --test differential
  cargo test -p bcinr-cmca --test hostile_mutants
  cargo test -p bcinr-cmca --test compile_fail_tests
  cargo check -p bcinr-cmca-audit-harness
  cargo test -p bcinr-pddl --all-features
  cargo test -p bcinr-powl --all-features
  cargo test -p bcinr-powl --all-features powl2
  cargo test -p bcinr-powl --all-features scheduler_v2
  cargo test -p bcinr-powl-receipt --all-features
  cargo test -p bcinr-powl-receipt --all-features execution_v2
  cargo test --workspace --all-features --all-targets --no-fail-fast
} 2>&1 | tee integration-tests.log

rm -f .github/workflows/integration-semantic-v2.yml
rm -f .github/workflows/integration-semantic-v3.yml
rm -f .github/workflows/integration-semantic-v4.yml
rm -f .github/workflows/integration-materialize.yml
rm -f .github/workflows/integration-execute.yml
rm -f scripts/complete_semantic_integration.py
rm -f scripts/complete-semantic-integration.sh
rm -f scripts/adapt_semantic_integration.py
rm -f scripts/run-semantic-integration-v4.sh
rm -f integration-check.log integration-tests.log

git config user.name 'OpenAI Integration Agent'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git add -A
git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py'
git commit -m 'merge: complete semantic recovery integration'
git push origin HEAD:"$BRANCH"
