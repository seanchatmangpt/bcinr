#!/usr/bin/env bash
# scripts/gates/block-generated-edit.sh
#
# Level-1 immediate-blocker gate (V26_7_17_HOOK_SPEC.md L1-2).
# Protects the generator-authoritative invariant: hand edits to generated
# output are never a valid source of truth; the generator (and its declared
# ontology input) is the only authoritative producer of files under
# crates/bcinr-cmca/src/generated/.
#
# Input contract:
#   Claude Code invokes PreToolUse "command" hooks for Edit/Write with a JSON
#   payload on stdin containing { "tool_input": { "file_path": "..." } }.
#   For standalone/unit-test invocation this script also accepts the target
#   path as $1.
#
# Exit code contract:
#   0  -> allow
#   2  -> block ("deny" exit code for PreToolUse hooks)
#
# Best-effort only: path-pattern match plus a check for an existing
# "DO NOT EDIT"/"GENERATED" header in the file's current on-disk content (if
# any). This is a single grep/path check -- no compilation, no workspace scan.

set -euo pipefail

GATE_NAME="cmca/rdf-generation.md runtime/producer boundary"
BLOCK_MSG="BLOCKED: direct edit to generated file (gate: ${GATE_NAME})"

read_file_path() {
  if [[ $# -ge 1 && -n "${1:-}" ]]; then
    printf '%s' "$1"
    return 0
  fi

  if [[ ! -t 0 ]]; then
    local payload
    payload="$(cat)"
    if [[ -n "$payload" ]]; then
      if command -v jq >/dev/null 2>&1; then
        local extracted
        extracted="$(printf '%s' "$payload" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)"
        if [[ -n "$extracted" ]]; then
          printf '%s' "$extracted"
          return 0
        fi
      fi
      printf '%s' "$payload"
      return 0
    fi
  fi

  printf ''
}

TARGET="$(read_file_path "${1:-}")"

if [[ -z "$TARGET" ]]; then
  exit 0
fi

# Normalize: strip a leading repo-absolute prefix down to the
# repository-relative form so the match works regardless of cwd.
REL="${TARGET#*crates/bcinr-cmca/src/generated/}"
if [[ "$REL" != "$TARGET" ]]; then
  MATCHES_PATTERN=1
else
  MATCHES_PATTERN=0
fi

HAS_GENERATED_HEADER=0
if [[ -f "$TARGET" ]]; then
  # Only the file's own header (first few lines, as a comment) counts --
  # this avoids false positives on files (like generator.py itself) whose
  # body merely contains the string "DO NOT EDIT" as a literal it writes
  # into the generated output it produces.
  if head -n 5 "$TARGET" 2>/dev/null | grep -qE '^[[:space:]]*(//|#).*(DO NOT EDIT|GENERATED)'; then
    HAS_GENERATED_HEADER=1
  fi
fi

if [[ "$MATCHES_PATTERN" -eq 1 || "$HAS_GENERATED_HEADER" -eq 1 ]]; then
  echo "$BLOCK_MSG" >&2
  echo "path: $TARGET" >&2
  exit 2
fi

exit 0
