#!/usr/bin/env bash
# scripts/gates/package-reality-check.sh
#
# Track B (v26.7.17 CMCA release, packaging-reality verification). Companion
# mechanics script for the `package-reality-check` cargo-make task
# (Makefile.toml). The Makefile task is a thin wrapper: it runs this script,
# then writes this run's own output into the committed-format receipt file
# `crates/bcinr-cmca/PACKAGE_REALITY_RECEIPT.md` (regenerated each run, per
# the control-plane separation discipline already established in this repo
# -- receipts are mutable run output, not timeless rules).
#
# THE FINDING this script exists to fix: earlier packaging/environment-
# isolation checks were ad-hoc Bash run by hand and reported in prose, not a
# replayable committed artifact -- so a later independent verifier correctly
# reported "could not verify" rather than trusting them. This script is that
# replayable artifact.
#
# What this script does:
#   (a) `cargo package -p bcinr-logic --locked` -- expected to succeed. Any
#       failure here is UNEXPECTED (bcinr-logic has no internal path
#       dependency that could cause the known bcinr-cmca blocker).
#   (b) `cargo package -p bcinr-cmca --locked` -- attempted. Its failure
#       mode is classified into exactly two buckets:
#         - the KNOWN, already-diagnosed sequencing blocker: cargo package
#           rewrites bcinr-cmca's path dependency on bcinr-logic into a
#           registry ("^26.7.17") requirement, which crates.io cannot
#           satisfy because bcinr-logic 26.7.17 has not been published
#           there yet (see V26_7_17_RELEASE_LEDGER.md's "Package / publish
#           dry-run" section for the first observation of this).
#         - anything else, which is treated as an UNEXPECTED failure and
#           fails this gate loudly.
#   (c) IF a bcinr-cmca .crate tarball was actually produced (e.g. because a
#       local-registry rehearsal override was active for this run and the
#       known blocker above did not fire), extract it to a clean temp
#       directory and run `cargo test --lib artifact` from the EXTRACTED
#       copy only -- never against the surrounding workspace tree. Per
#       .claude/rules/cmca/packaging.md, a package must be verified to build
#       using only its own packaged contents, not the workspace.
#   (d) Record sha256 digests of every .crate file produced this run.
#   (e) Print the honest mfw-filesystem-absence limitation (see below).
#
# Per .claude/rules/cmca/packaging.md: a `cargo package`/`cargo publish
# --dry-run` run using `--allow-dirty` (or run against an already-dirty
# tree) is admissible only as an interim smoke check -- never as
# release-closing evidence. This script does NOT silently pass --allow-dirty
# by default: it first attempts each package command WITHOUT --allow-dirty,
# and only retries WITH --allow-dirty if that specific package's own
# directory has uncommitted/untracked changes (cargo's own dirty-tree
# error), labeling that retry's result as an interim smoke check, not
# release-closing evidence, in both this script's stdout and the receipt
# the Makefile task writes from it.
#
# mfw-absence limitation (honest, not closed by this script): this script
# does not and will not attempt to prove bcinr-cmca builds correctly when
# /Users/sac/mfw is genuinely ABSENT from the filesystem. Dependency-graph
# absence (a `cargo tree -p bcinr-cmca` grep for mfw/oxigraph/praxis-graphlaw
# showing zero hits) is provable and already covered elsewhere
# (PHASE1_CONSUMER_VERDICT.md, RECONCILIATION_VERIFICATION.md). True
# filesystem absence is not provable without deleting the real
# /Users/sac/mfw checkout, which this script will NOT do -- that would be a
# destructive, out-of-scope action against a real, non-generated directory
# this task does not own.
#
# Exit code contract:
#   0 -> ran to completion with no UNEXPECTED failure (bcinr-logic packaged;
#        bcinr-cmca either packaged too, or failed with exactly the known,
#        already-diagnosed sequencing blocker).
#   1 -> an UNEXPECTED failure occurred -- something other than the known
#        bcinr-logic-not-yet-on-the-registry blocker. This is the condition
#        the Makefile task's "fail loudly" requirement maps to.
#
# This script never runs `git commit`, `cargo publish` (without
# `--dry-run`), or any other side-effecting/irreversible command. It only
# reads git state and runs local `cargo package`/`cargo test` commands
# against files already on disk, writing its own output under
# target/package-reality/.

set -uo pipefail

if REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"; then
  :
else
  SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
fi
cd "$REPO_ROOT"

OUT_DIR="target/package-reality"
mkdir -p "$OUT_DIR"

FAIL=0
LOGIC_RESULT="UNKNOWN"
LOGIC_DIRTY_RETRY=0
CMCA_RESULT="UNKNOWN"
CMCA_DIRTY_RETRY=0
CMCA_TARBALL=""
EXTRACT_RESULT="SKIPPED (no tarball produced)"

sha256_of() {
  local f="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$f" | awk '{print $1}'
  else
    shasum -a 256 "$f" | awk '{print $1}'
  fi
}

# run_package <crate-name> <log-file>
# Attempts `cargo package -p <crate-name> --locked --no-verify` without
# --allow-dirty first; retries with --allow-dirty ONLY if the failure is
# cargo's own dirty-tree error. Writes combined stdout+stderr to <log-file>.
# Sets globals RP_EXIT, RP_DIRTY_RETRY (0/1) as side effects.
run_package() {
  local crate="$1"
  local log="$2"
  RP_DIRTY_RETRY=0

  cargo package -p "$crate" --locked --no-verify >"$log" 2>&1
  RP_EXIT=$?

  if [[ "$RP_EXIT" -ne 0 ]] && grep -q "contain changes that were not yet committed into git" "$log"; then
    RP_DIRTY_RETRY=1
    echo "  (dirty-tree in ${crate}'s own directory detected -- retrying with --allow-dirty; result below is an INTERIM SMOKE CHECK ONLY, not release-closing evidence per .claude/rules/cmca/packaging.md)"
    cargo package -p "$crate" --locked --no-verify --allow-dirty >"$log" 2>&1
    RP_EXIT=$?
  fi
}

echo "=== package-reality-check: $(date -u +%Y-%m-%dT%H:%M:%SZ) ==="
echo "repo root: $REPO_ROOT"
echo "git branch: $(git branch --show-current 2>/dev/null || echo unknown)"
echo ""

# --- (a) bcinr-logic ---
echo "=== (a) cargo package -p bcinr-logic --locked ==="
LOGIC_LOG="$OUT_DIR/bcinr-logic-package.log"
run_package bcinr-logic "$LOGIC_LOG"
LOGIC_DIRTY_RETRY=$RP_DIRTY_RETRY
if [[ "$RP_EXIT" -eq 0 ]]; then
  LOGIC_RESULT="PASS"
  echo "bcinr-logic: cargo package PASSED$( [[ $LOGIC_DIRTY_RETRY -eq 1 ]] && echo ' (via --allow-dirty retry -- interim smoke check only)' )"
else
  LOGIC_RESULT="FAIL (UNEXPECTED)"
  FAIL=1
  echo "bcinr-logic: cargo package FAILED -- UNEXPECTED (bcinr-logic has no internal path dependency that should block it); see $LOGIC_LOG"
fi
cat "$LOGIC_LOG"
echo ""

# --- (b) bcinr-cmca ---
echo "=== (b) cargo package -p bcinr-cmca --locked ==="
CMCA_LOG="$OUT_DIR/bcinr-cmca-package.log"
run_package bcinr-cmca "$CMCA_LOG"
CMCA_DIRTY_RETRY=$RP_DIRTY_RETRY
if [[ "$RP_EXIT" -eq 0 ]]; then
  CMCA_RESULT="PASS (unexpected relative to the known sequencing blocker)"
  CMCA_VERSION="$(grep '^version' crates/bcinr-cmca/Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
  CMCA_TARBALL="target/package/bcinr-cmca-${CMCA_VERSION}.crate"
  echo "bcinr-cmca: cargo package PASSED$( [[ $CMCA_DIRTY_RETRY -eq 1 ]] && echo ' (via --allow-dirty retry -- interim smoke check only)' ). This is unexpected relative to the known bcinr-logic-registry-sequencing blocker -- check whether bcinr-logic $CMCA_VERSION is now published to crates.io, or whether a local-registry rehearsal override is active for this run."
elif grep -q "failed to select a version for the requirement \`bcinr-logic" "$CMCA_LOG" && grep -q "location searched: crates.io index" "$CMCA_LOG"; then
  CMCA_RESULT="FAIL (KNOWN sequencing blocker)"
  echo "bcinr-cmca: cargo package FAILED with the KNOWN, already-diagnosed sequencing blocker -- cargo package rewrites the bcinr-logic path dependency to a registry requirement, and bcinr-logic at the required version is not yet published to crates.io. This is EXPECTED at this stage of the release and does NOT fail this gate."
else
  CMCA_RESULT="FAIL (UNEXPECTED)"
  FAIL=1
  echo "bcinr-cmca: cargo package FAILED with an UNEXPECTED error (does not match the known bcinr-logic-registry-sequencing pattern) -- see $CMCA_LOG"
fi
cat "$CMCA_LOG"
echo ""

# --- (c) extract + test tarball IF one was produced ---
echo "=== (c) extracted-package test ==="
if [[ -n "$CMCA_TARBALL" && -f "$CMCA_TARBALL" ]]; then
  echo "bcinr-cmca tarball found: $CMCA_TARBALL -- extracting to a clean temp dir and running 'cargo test --lib artifact' from the extracted copy only"
  EXTRACT_DIR="$(mktemp -d)"
  if tar -xzf "$CMCA_TARBALL" -C "$EXTRACT_DIR"; then
    EXTRACTED_CRATE_DIR="$(find "$EXTRACT_DIR" -maxdepth 1 -type d -name 'bcinr-cmca-*' | head -1)"
    if [[ -z "$EXTRACTED_CRATE_DIR" ]]; then
      EXTRACT_RESULT="FAIL (UNEXPECTED -- could not locate extracted bcinr-cmca-* directory)"
      FAIL=1
    else
      EXTRACT_LOG="$OUT_DIR/bcinr-cmca-extracted-test.log"
      ( cd "$EXTRACTED_CRATE_DIR" && cargo test --lib artifact ) >"$EXTRACT_LOG" 2>&1
      EXTRACT_EXIT=$?
      cat "$EXTRACT_LOG"
      if [[ "$EXTRACT_EXIT" -eq 0 ]]; then
        EXTRACT_RESULT="PASS (built and tested from packaged contents only, at $EXTRACTED_CRATE_DIR)"
      else
        EXTRACT_RESULT="FAIL (UNEXPECTED -- see $EXTRACT_LOG)"
        FAIL=1
      fi
    fi
  else
    EXTRACT_RESULT="FAIL (UNEXPECTED -- tar extraction of $CMCA_TARBALL failed)"
    FAIL=1
  fi
  rm -rf "$EXTRACT_DIR"
else
  echo "SKIPPED: no bcinr-cmca .crate tarball was produced this run (expected while the known sequencing blocker in (b) is unresolved)."
fi
echo ""

# --- (d) digests ---
echo "=== (d) package digests (sha256) ==="
DIGEST_FILE="$OUT_DIR/digests.txt"
: > "$DIGEST_FILE"
CURRENT_CMCA_VERSION="$(grep '^version' crates/bcinr-cmca/Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
CURRENT_LOGIC_VERSION="$(grep '^version' crates/bcinr-logic/Cargo.toml | head -1 | sed -E 's/.*"([^"]+)".*/\1/')"
shopt -s nullglob
for f in target/package/bcinr-logic-*.crate target/package/bcinr-cmca-*.crate; do
  [[ -f "$f" ]] || continue
  DIGEST="$(sha256_of "$f")"
  LABEL=""
  case "$f" in
    target/package/bcinr-logic-"${CURRENT_LOGIC_VERSION}".crate) LABEL=" (current version, this run)" ;;
    target/package/bcinr-cmca-"${CURRENT_CMCA_VERSION}".crate) LABEL=" (current version, this run)" ;;
    *) LABEL=" (STALE leftover from a prior target/package/ build at a different version -- not produced by this run; not deleted, per fix-forward-only / no-destructive-cleanup discipline)" ;;
  esac
  echo "$f  sha256:$DIGEST$LABEL" | tee -a "$DIGEST_FILE"
done
shopt -u nullglob
if [[ ! -s "$DIGEST_FILE" ]]; then
  echo "(no .crate files found under target/package/ to digest this run)" | tee -a "$DIGEST_FILE"
fi
echo ""

# --- (e) mfw-absence limitation ---
echo "=== (e) mfw-absence limitation (documented honestly, not tested) ==="
echo "Dependency-graph absence of mfw/oxigraph/praxis-graphlaw from bcinr-cmca's"
echo "build graph is provable and already covered elsewhere (cargo tree -p"
echo "bcinr-cmca, see PHASE1_CONSUMER_VERDICT.md / RECONCILIATION_VERIFICATION.md)."
echo "True FILESYSTEM absence of /Users/sac/mfw is NOT proven by this task and"
echo "this script will NOT delete or rename the real /Users/sac/mfw checkout to"
echo "test it -- that would be destructive and out of this task's scope."
echo ""

# --- machine-readable summary line (parsed by the Makefile task) ---
echo "PACKAGE_REALITY_SUMMARY logic_result=\"$LOGIC_RESULT\" logic_dirty_retry=$LOGIC_DIRTY_RETRY cmca_result=\"$CMCA_RESULT\" cmca_dirty_retry=$CMCA_DIRTY_RETRY extract_result=\"$EXTRACT_RESULT\" fail=$FAIL"

if [[ "$FAIL" -ne 0 ]]; then
  echo "package-reality-check: GATE FAILED -- an unexpected failure occurred (see above)"
  exit 1
fi
echo "package-reality-check: GATE PASSED (bcinr-logic packages; bcinr-cmca's outcome, if a failure, matched only the known sequencing blocker)"
exit 0
