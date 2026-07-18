#!/usr/bin/env bash
# Must-pass / must-block test for scripts/gates/block-generated-edit.sh
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
GATE="${SCRIPT_DIR}/../block-generated-edit.sh"

fail=0

check() {
  local desc="$1" path="$2" expect_block="$3"
  "$GATE" "$path" >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
  local rc=$?
  if [[ "$expect_block" == "block" ]]; then
    if [[ "$rc" -eq 0 ]]; then
      echo "FAIL: expected BLOCK for [$desc] but got exit 0: $path"
      fail=1
    else
      echo "PASS (blocked, exit $rc): $desc"
    fi
  else
    if [[ "$rc" -ne 0 ]]; then
      echo "FAIL: expected PASS for [$desc] but got exit $rc: $path"
      cat /tmp/gate-err.$$
      fail=1
    else
      echo "PASS (allowed, exit 0): $desc"
    fi
  fi
  rm -f /tmp/gate-out.$$ /tmp/gate-err.$$
}

# Must-pass: editing the generator source itself, outside src/generated/
check "edit generator.py" "${REPO_ROOT}/crates/bcinr-cmca/generator.py" allow

# Must-block: direct edit to a generated file
check "edit generated case_studies.rs" "${REPO_ROOT}/crates/bcinr-cmca/src/generated/case_studies.rs" block

exit $fail
