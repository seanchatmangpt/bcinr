#!/usr/bin/env bash
set -uo pipefail

BRANCH=agent/semantic-three-way-integration-v26.7.24
export EXPECTED_REMOTE_HEAD
EXPECTED_REMOTE_HEAD=$(git rev-parse HEAD)

python3 - <<'PY'
from pathlib import Path

path = Path("scripts/run-semantic-integration-v4.sh")
text = path.read_text()
old = """rm -f scripts/run-semantic-integration-v4.sh
rm -f integration-check.log integration-tests.log

git config user.name 'OpenAI Integration Agent'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git add -A
git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py'
git commit -m 'merge: complete semantic recovery integration'
git push origin HEAD:\"$BRANCH\"
"""
new = """rm -f scripts/run-semantic-integration-v4.sh
rm -f scripts/publish-semantic-integration-v5.sh
rm -f integration-check.log integration-tests.log

git config user.name 'OpenAI Integration Agent'
git config user.email '41898282+github-actions[bot]@users.noreply.github.com'
git add -A

if ! git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py' > /tmp/diff-check.out 2>&1; then
  python3 - <<'PYFIX'
from pathlib import Path

for line in Path('/tmp/diff-check.out').read_text().splitlines():
    if ': trailing whitespace.' not in line:
        continue
    candidate = Path(line.rsplit(':', 2)[0])
    try:
        source = candidate.read_text()
    except (OSError, UnicodeDecodeError):
        continue
    final_newline = '\\n' if source.endswith('\\n') else ''
    candidate.write_text('\\n'.join(row.rstrip(' \\t') for row in source.splitlines()) + final_newline)
PYFIX
  git add -A
  git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py'
fi

test -n \"$(git diff --cached --name-only)\"
git commit -m 'merge: complete semantic recovery integration'
git push --force-with-lease=\"refs/heads/$BRANCH:$EXPECTED_REMOTE_HEAD\" origin \"HEAD:refs/heads/$BRANCH\"
"""
if old not in text:
    raise SystemExit("publication tail did not match expected V4 script")
path.write_text(text.replace(old, new))
PY

set +e
bash scripts/run-semantic-integration-v4.sh
status=$?
set -e

{
  echo "exit_status=$status"
  echo "expected_remote_head=$EXPECTED_REMOTE_HEAD"
  echo "current_head=$(git rev-parse HEAD 2>/dev/null)"
  echo "--- git status --short ---"
  git status --short
  echo "--- cached diff check ---"
  git diff --cached --check -- ':!crates/bcinr-cmca/quarantine/legacy-generator/generator.py' || true
  echo "--- last commit ---"
  git log -1 --oneline
} > publication-diagnostics.log 2>&1

exit "$status"
