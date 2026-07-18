#!/usr/bin/env bash
# scripts/gates/block-destructive-git.sh
#
# Level-1 immediate-blocker gate (V26_7_17_HOOK_SPEC.md L1-1).
# Protects the fix-forward-only invariant (CLAUDE.md "CRITICAL: FIX FORWARD ONLY"):
# commits are immutable, history is never rewritten, no destructive git operation
# runs without explicit human override outside this hook path.
#
# Input contract:
#   Claude Code invokes PreToolUse "command" hooks with a JSON payload on stdin
#   containing (at least) { "tool_name": "...", "tool_input": { "command": "..." } }.
#   For standalone/manual invocation this script also accepts the candidate
#   command string as $1, so it can be unit-tested without fabricating JSON.
#
# Exit code contract:
#   0  -> allow (command does not match a destructive pattern)
#   2  -> block (Claude Code's documented PreToolUse "deny" exit code); a message
#         is written to stderr naming the gate, per this file's own pattern list
#         (no inline duplication of gate prose elsewhere).
#
# This script owns the pattern list. It does not compile, does not shell out to
# git, and does not scan the workspace -- pure text pattern matching only, to
# stay within the Level-1 cost budget (grep/path-match only, no perceptible
# latency).

set -euo pipefail

GATE_NAME="20-repository-safety"
BLOCK_MSG="BLOCKED: destructive git operation (gate: ${GATE_NAME})"

read_command() {
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
        extracted="$(printf '%s' "$payload" | jq -r '.tool_input.command // empty' 2>/dev/null || true)"
        if [[ -n "$extracted" ]]; then
          printf '%s' "$extracted"
          return 0
        fi
      fi
      # Fall back to treating the raw stdin payload itself as the command
      # string (covers manual `echo "<cmd>" | script` test invocations).
      printf '%s' "$payload"
      return 0
    fi
  fi

  printf ''
}

CMD="$(read_command "${1:-}")"

if [[ -z "$CMD" ]]; then
  # Nothing to inspect -> nothing to block.
  exit 0
fi

is_destructive() {
  local cmd="$1"

  # git reset --hard (any ref/no ref)
  if [[ "$cmd" =~ git[[:space:]]+reset[[:space:]]+.*--hard ]]; then
    return 0
  fi

  # git clean -fdx / -fd -x / -xfd / any combination of f, d, x flags together
  if [[ "$cmd" =~ git[[:space:]]+clean[[:space:]] ]]; then
    if [[ "$cmd" =~ git[[:space:]]+clean[[:space:]]+.*-[a-zA-Z]*f[a-zA-Z]*d ]] || \
       [[ "$cmd" =~ git[[:space:]]+clean[[:space:]]+.*-[a-zA-Z]*d[a-zA-Z]*f ]] || \
       [[ "$cmd" =~ git[[:space:]]+clean[[:space:]]+.*-[a-zA-Z]*x ]]; then
      return 0
    fi
  fi

  # push --force / -f, but NOT --force-with-lease
  if [[ "$cmd" =~ git[[:space:]]+push ]]; then
    if [[ "$cmd" =~ --force-with-lease ]]; then
      : # explicitly allowed
    elif [[ "$cmd" =~ --force([[:space:]]|$) ]] || [[ "$cmd" =~ [[:space:]]-f([[:space:]]|$) ]]; then
      return 0
    fi
  fi

  # rebase or filter-branch/filter-repo touching a public/pushed ref
  # (main, master, or origin/<anything>) -- conservative: any rebase/filter
  # invocation that mentions origin/, main, or master is treated as rewriting
  # already-pushed history.
  if [[ "$cmd" =~ git[[:space:]]+rebase ]]; then
    if [[ "$cmd" =~ origin/ ]] || [[ "$cmd" =~ [[:space:]]main([[:space:]]|$) ]] || [[ "$cmd" =~ [[:space:]]master([[:space:]]|$) ]]; then
      return 0
    fi
  fi

  if [[ "$cmd" =~ filter-branch ]] || [[ "$cmd" =~ filter-repo ]]; then
    return 0
  fi

  return 1
}

if is_destructive "$CMD"; then
  echo "$BLOCK_MSG" >&2
  echo "command: $CMD" >&2
  exit 2
fi

exit 0
