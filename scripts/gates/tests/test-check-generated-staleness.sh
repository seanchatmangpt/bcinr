#!/usr/bin/env bash
# Must-pass / must-block test for scripts/gates/check-generated-staleness.sh
#
# Runs the real gate against the live repo (must-pass, since checked-in
# generated files should currently match their declared sources), then
# reruns it against a scratch copy of the repo subtree with a tampered
# ontology file to force a digest mismatch (must-block).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
GATE="${SCRIPT_DIR}/../check-generated-staleness.sh"

fail=0

# Must-pass: current repo state, digests should match.
( cd "$REPO_ROOT" && "$GATE" ) >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
rc=$?
if [[ "$rc" -ne 0 ]]; then
  echo "FAIL: expected PASS against live repo state, got exit $rc"
  cat /tmp/gate-err.$$
  fail=1
else
  echo "PASS (exit 0): live repo digests match"
fi
rm -f /tmp/gate-out.$$ /tmp/gate-err.$$

# Must-block: copy the cmca crate subtree, tamper the ontology source so its
# fresh digest no longer matches the embedded RDF_INPUT_DIGEST, then rerun
# the gate against that scratch copy (never touching the real repo).
SCRATCH="$(mktemp -d)"
mkdir -p "${SCRATCH}/crates"
cp -R "${REPO_ROOT}/crates/bcinr-cmca" "${SCRATCH}/crates/bcinr-cmca"
echo "# tampered for staleness test $(date +%s)" >> "${SCRATCH}/crates/bcinr-cmca/ontology/cmca-rdf.ttl"

# The gate resolves REPO_ROOT via `git rev-parse --show-toplevel` first; the
# scratch dir isn't a git repo, so it falls back to resolving relative to the
# script's own location -- which still points at the real repo. To exercise
# the scratch tree we instead invoke the gate with a copy of itself placed at
# the equivalent scripts/gates path inside the scratch tree.
mkdir -p "${SCRATCH}/scripts/gates"
cp "$GATE" "${SCRATCH}/scripts/gates/check-generated-staleness.sh"
chmod +x "${SCRATCH}/scripts/gates/check-generated-staleness.sh"

( cd "$SCRATCH" && ./scripts/gates/check-generated-staleness.sh ) >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
rc=$?
if [[ "$rc" -eq 0 ]]; then
  echo "FAIL: expected BLOCK against tampered ontology source, got exit 0"
  fail=1
else
  echo "PASS (blocked, exit $rc): tampered ontology detected as stale"
fi
rm -f /tmp/gate-out.$$ /tmp/gate-err.$$
rm -rf "$SCRATCH"

exit $fail
