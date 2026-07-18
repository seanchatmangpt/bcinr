#!/usr/bin/env bash
# Must-pass (no warning) / must-block(warning present) test for
# scripts/gates/check-authority-surface.sh
#
# This gate never fails (always exits 0); "must-block" here means "must emit
# the WARNING to stderr" per the task's non-blocking design.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../../.." && pwd)"
GATE="${SCRIPT_DIR}/../check-authority-surface.sh"

fail=0
TMPDIR_T="$(mktemp -d)"

# Must-pass: a file with only a private field on an authority type -> no warning, exit 0
cat > "${TMPDIR_T}/private_field.rs" <<'EOF'
pub struct CertificateReceipt {
    inner_hash: [u8; 32],
    count: u32,
}
EOF

"$GATE" "${TMPDIR_T}/private_field.rs" >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
rc=$?
if [[ "$rc" -ne 0 ]]; then
  echo "FAIL: check-authority-surface.sh must never block (exit $rc)"
  fail=1
elif grep -q "WARNING" /tmp/gate-err.$$; then
  echo "FAIL: expected no WARNING for private-field-only struct, got:"
  cat /tmp/gate-err.$$
  fail=1
else
  echo "PASS (no warning, exit 0): private field only"
fi
rm -f /tmp/gate-out.$$ /tmp/gate-err.$$

# Must-flag: a new pub field on an authority type -> WARNING on stderr, still exit 0
cat > "${TMPDIR_T}/pub_field.rs" <<'EOF'
pub struct CertificateReceipt {
    pub inner_hash: [u8; 32],
    count: u32,
}
EOF

"$GATE" "${TMPDIR_T}/pub_field.rs" >/tmp/gate-out.$$ 2>/tmp/gate-err.$$
rc=$?
if [[ "$rc" -ne 0 ]]; then
  echo "FAIL: check-authority-surface.sh must never block (exit $rc)"
  fail=1
elif ! grep -q "WARNING" /tmp/gate-err.$$; then
  echo "FAIL: expected WARNING for new pub field, got no warning:"
  cat /tmp/gate-err.$$
  fail=1
else
  echo "PASS (warning present, exit 0): new pub field flagged"
fi
rm -f /tmp/gate-out.$$ /tmp/gate-err.$$

rm -rf "${TMPDIR_T}"
exit $fail
