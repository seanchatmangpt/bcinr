#!/usr/bin/env bash
# Must-pass / must-block test for scripts/gates/block-destructive-git.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GATE="${SCRIPT_DIR}/../block-destructive-git.sh"

fail=0

check() {
  local desc="$1" cmd="$2" expect_block="$3"
  "$GATE" "$cmd" >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
  local rc=$?
  if [[ "$expect_block" == "block" ]]; then
    if [[ "$rc" -eq 0 ]]; then
      echo "FAIL: expected BLOCK for [$desc] but got exit 0: $cmd"
      fail=1
    else
      echo "PASS (blocked, exit $rc): $desc"
    fi
  else
    if [[ "$rc" -ne 0 ]]; then
      echo "FAIL: expected PASS for [$desc] but got exit $rc: $cmd"
      cat /tmp/gate-err.$$
      fail=1
    else
      echo "PASS (allowed, exit 0): $desc"
    fi
  fi
  rm -f /tmp/gate-out.$$ /tmp/gate-err.$$
}

# Must-pass cases
check "commit"                         'git commit -m "fix: something"'          allow
check "revert"                         'git revert abc1234'                      allow
check "force-with-lease to feature br" 'git push --force-with-lease origin recovery/cmca-v26.7.17-c2' allow

# Must-block cases
check "reset --hard"   'git reset --hard HEAD~3'      block
check "clean -fdx"     'git clean -fdx'                block
check "push --force main" 'git push --force origin main' block

exit $fail
