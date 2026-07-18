#!/usr/bin/env bash
# scripts/gates/check-generated-staleness.sh
#
# Level-1 immediate-blocker gate (V26_7_17_HOOK_SPEC.md L1-4).
# Protects the generator/output correspondence invariant: every file under
# crates/bcinr-cmca/src/generated/ embeds a digest of the ontology/generator
# source it was produced from; that digest must match the current declared
# source at all times the generated file is read as authoritative.
#
# This script is repository-relative: it resolves paths from the git
# toplevel (or, if not in a git checkout, from its own location) rather than
# hardcoding any machine-specific absolute path.
#
# Exit code contract:
#   0 -> match (or post-migration state: generator.py/ontology no longer
#        exist, so there is nothing to be stale relative to)
#   2 -> BLOCKED: generated output digest no longer matches its source
#        (gate: cmca/rdf-generation.md determinism)

set -euo pipefail

GATE_NAME="cmca/rdf-generation.md determinism"
BLOCK_MSG="BLOCKED: generated output digest no longer matches its source (gate: ${GATE_NAME})"

# Resolve repo root without hardcoding a username-specific absolute path.
if REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  :
else
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
fi

CMCA_DIR="${REPO_ROOT}/crates/bcinr-cmca"
GENERATOR="${CMCA_DIR}/generator.py"

# generated_file:ontology_source pairs this script knows about.
declare -a PAIRS=(
  "${CMCA_DIR}/src/generated/case_studies.rs:${CMCA_DIR}/ontology/cmca-rdf.ttl"
  "${CMCA_DIR}/src/generated/generalization.rs:${CMCA_DIR}/ontology/generalization.ttl"
)

sha256_of() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  else
    shasum -a 256 "$f" | awk '{print $1}'
  fi
}

# Post-migration state: generator no longer exists at all -> nothing to
# verify staleness against, so pass (per spec's explicit carve-out).
if [[ ! -f "$GENERATOR" ]]; then
  exit 0
fi

STALE=0
for PAIR in "${PAIRS[@]}"; do
  GENFILE="${PAIR%%:*}"
  ONTOLOGY="${PAIR##*:}"

  if [[ ! -f "$GENFILE" ]]; then
    continue
  fi
  if [[ ! -f "$ONTOLOGY" ]]; then
    # Ontology source itself gone (post-migration for this pair) -> skip.
    continue
  fi

  EMBEDDED_RDF_DIGEST="$(grep -oE 'RDF_INPUT_DIGEST: &str = "[0-9a-f]+"' "$GENFILE" | grep -oE '[0-9a-f]{32,}' || true)"
  EMBEDDED_GEN_DIGEST="$(grep -oE 'GENERATOR_SOURCE_DIGEST: &str = "[0-9a-f]+"' "$GENFILE" | grep -oE '[0-9a-f]{32,}' || true)"

  FRESH_RDF_DIGEST="$(sha256_of "$ONTOLOGY")"
  FRESH_GEN_DIGEST="$(sha256_of "$GENERATOR")"

  if [[ -z "$EMBEDDED_RDF_DIGEST" || -z "$EMBEDDED_GEN_DIGEST" ]]; then
    # Can't find embedded digests -> treat as stale (fail closed).
    echo "note: could not find embedded digest constants in $GENFILE" >&2
    STALE=1
    continue
  fi

  if [[ "$EMBEDDED_RDF_DIGEST" != "$FRESH_RDF_DIGEST" ]]; then
    echo "note: RDF_INPUT_DIGEST mismatch for $GENFILE (embedded=$EMBEDDED_RDF_DIGEST fresh=$FRESH_RDF_DIGEST, source=$ONTOLOGY)" >&2
    STALE=1
  fi

  if [[ "$EMBEDDED_GEN_DIGEST" != "$FRESH_GEN_DIGEST" ]]; then
    echo "note: GENERATOR_SOURCE_DIGEST mismatch for $GENFILE (embedded=$EMBEDDED_GEN_DIGEST fresh=$FRESH_GEN_DIGEST, source=$GENERATOR)" >&2
    STALE=1
  fi
done

if [[ "$STALE" -eq 1 ]]; then
  echo "$BLOCK_MSG" >&2
  exit 2
fi

exit 0
