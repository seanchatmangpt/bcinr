#!/usr/bin/env bash
# scripts/gates/check-authority-surface.sh
#
# Level-1 gate, non-blocking variant of V26_7_17_HOOK_SPEC.md L1-3
# (New public field or safe-construction path on an authority-bearing type).
#
# IMPORTANT / SCOPE NOTE: this is a best-effort grep-level check, NOT a full
# AST audit. It cannot see whether a "pub " token is newly added by the
# current edit vs. already present on disk -- it can only report that a
# public field currently exists inside the textual body of a known
# authority-bearing struct in the target file. A full before/after AST diff
# of field lists and impl-block signatures (as the spec's L1-3 ultimately
# calls for) is the job of the object-code-audit skill / turing-machine
# agent, not this script. Treat this script's output as a prompt for human
# or turing-machine review, never as a substitute for that audit.
#
# Input contract: target file path as $1, or via stdin JSON
# { "tool_input": { "file_path": "..." } } (PostToolUse convention).
#
# Exit code contract: ALWAYS 0 (never blocks). On a match, prints a WARNING
# to stderr; the caller (hook) is expected to surface stderr to the operator
# without failing the tool call.

set -euo pipefail

GATE_NAME="30-authority-separation.md"
WARN_MSG="WARNING: possible new public field on authority type (gate: ${GATE_NAME})"

# Authority-bearing type names this best-effort check knows about (per the
# task's fixed list; not derived from a doc-comment marker scan, which would
# require the AST-level audit called for in the spec).
AUTHORITY_TYPES=(
  "CertificateReceipt"
  "CertifiedLearning"
  "AdmittedControlState"
  "EnvelopeReceipt"
  "OutcomeReceipt"
  "CertifiedSelectionOnly"
  "AdaptiveUpdate"
)

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

if [[ -z "$TARGET" || ! -f "$TARGET" ]]; then
  exit 0
fi

FOUND=0
for TYPE in "${AUTHORITY_TYPES[@]}"; do
  # Extract the struct body: from "struct <Type>" (or "struct <Type><" for
  # generics) up to the first line containing a closing brace at column 0,
  # a crude but cheap single-file text scan (no AST parse).
  # Capture only the struct BODY (fields), never the "struct Foo {"
  # declaration line itself -- otherwise "pub struct Foo" would falsely
  # match as a "pub field".
  BODY="$(awk -v t="$TYPE" '
    $0 ~ "struct[ \t]+" t "([ \t]*<|[ \t]*\\{|$)" { capture=1; next }
    capture { print }
    capture && /^}/ { exit }
  ' "$TARGET" 2>/dev/null || true)"

  if [[ -n "$BODY" ]] && printf '%s\n' "$BODY" | grep -qE '^\s*pub[[:space:]]+[a-zA-Z_]'; then
    FOUND=1
    echo "note: possible pub field/member inside ${TYPE}" >&2
  fi
done

if [[ "$FOUND" -eq 1 ]]; then
  echo "$WARN_MSG" >&2
  echo "path: $TARGET" >&2
fi

exit 0
