#!/usr/bin/env bash
set -uo pipefail

# BCINR v26.7.26 executable verifier.
# Every ALIVE state is derived from an observed command exit code. The script
# never embeds expected test counts, benchmark timings, or mutation outcomes.

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RUN_ID="${BCINR_VERIFICATION_RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)-$$}"
EVIDENCE_ROOT="${BCINR_VERIFICATION_DIR:-${REPO_ROOT}/target/v26.7.26-verification/${RUN_ID}}"
LOG_DIR="${EVIDENCE_ROOT}/logs"
LEDGER="${EVIDENCE_ROOT}/ledger.tsv"
REPORT="${EVIDENCE_ROOT}/v26_7_26_verification_report.md"
mkdir -p "${LOG_DIR}"
printf 'phase\tcheck\tlabel\texit_code\tduration_seconds\ttest_summary\tlog\tcommand\n' > "${LEDGER}"

STARTED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
HEAD_SHA="$(git -C "${REPO_ROOT}" rev-parse HEAD 2>/dev/null || printf 'UNKNOWN')"
TREE_STATE="clean"
if ! git -C "${REPO_ROOT}" diff --quiet --ignore-submodules -- 2>/dev/null || \
   ! git -C "${REPO_ROOT}" diff --cached --quiet --ignore-submodules -- 2>/dev/null; then
    TREE_STATE="dirty"
fi

sanitize() {
    printf '%s' "$1" | tr '\t\r\n' '   '
}

summarize_tests() {
    python3 - "$1" <<'PY'
import re
import sys
from pathlib import Path
text = Path(sys.argv[1]).read_text(encoding="utf-8", errors="replace")
rows = re.findall(r"test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out", text)
if not rows:
    print("n/a")
else:
    totals = [sum(int(row[i]) for row in rows) for i in range(5)]
    print(f"passed={totals[0]},failed={totals[1]},ignored={totals[2]},measured={totals[3]},filtered={totals[4]}")
PY
}

run_check() {
    local phase="$1"
    local check="$2"
    local label="$3"
    local command="$4"
    local log="${LOG_DIR}/${check}.log"
    local start end rc duration summary

    start="$(date +%s)"
    printf '[RUN] phase=%s check=%s command=%s\n' "${phase}" "${check}" "${command}" | tee "${log}"
    (
        cd "${REPO_ROOT}" || exit 125
        bash -lc "${command}"
    ) >> "${log}" 2>&1
    rc=$?
    end="$(date +%s)"
    duration=$((end - start))
    summary="$(summarize_tests "${log}")"
    printf '[RECEIPT] exit=%s duration_seconds=%s tests=%s\n' "${rc}" "${duration}" "${summary}" | tee -a "${log}"
    printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$(sanitize "${phase}")" \
        "$(sanitize "${check}")" \
        "$(sanitize "${label}")" \
        "${rc}" \
        "${duration}" \
        "$(sanitize "${summary}")" \
        "$(sanitize "${log#${REPO_ROOT}/}")" \
        "$(sanitize "${command}")" >> "${LEDGER}"
    return 0
}

phase_status() {
    local phase="$1"
    python3 - "${LEDGER}" "${phase}" <<'PY'
import csv
import sys
from pathlib import Path
ledger, phase = sys.argv[1:]
rows = [row for row in csv.DictReader(Path(ledger).open(), delimiter="\t") if row["phase"] == phase]
if not rows:
    print("UNKNOWN")
elif any(int(row["exit_code"]) != 0 for row in rows):
    print("BLOCKED")
else:
    print("ALIVE")
PY
}

# Phase 1 — Temporal Core
run_check "1" "p1-structure" "Temporal source admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 1"
run_check "1" "p1-tests" "Temporal conditions and deadlines" \
    "cargo test -p bcinr-pddl --test temporal_conditions --test temporal_deadlines -- --nocapture"

# Phase 2 — Resource Intervals
run_check "2" "p2-structure" "Resource interval source admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 2"
run_check "2" "p2-tests" "Resource lease integration" \
    "cargo test -p bcinr-pddl --test resource_leases -- --nocapture"

# Phase 3 — Temporal Scheduler
run_check "3" "p3-structure" "Scheduler lifecycle source admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 3"
run_check "3" "p3-tests" "Scheduler lifecycle and resource conflicts" \
    "cargo test -p bcinr-powl --test scheduler_lifecycle --test scheduler_resource_conflict -- --nocapture"

# Phase 4 — Evidence / OCEL
run_check "4" "p4-structure" "Temporal evidence source admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 4"
run_check "4" "p4-tests" "OCEL temporal conformance" \
    "cargo test -p bcinr-powl --test ocel_temporal --test ocel_conformance_temporal -- --nocapture"

# Phase 5 — Swarm Scenarios
run_check "5" "p5-structure" "Ten-scenario and zero-LLM admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 5"
mapfile -t SWARM_TESTS < <(cd "${REPO_ROOT}" && printf '%s\n' crates/bcinr-powl/tests/usecase_swarm_*.rs 2>/dev/null | sort)
if [[ "${#SWARM_TESTS[@]}" -eq 0 || "${SWARM_TESTS[0]}" == *'*'* ]]; then
    run_check "5" "p5-tests-missing" "Swarm scenario execution" "false"
else
    for file in "${SWARM_TESTS[@]}"; do
        target="$(basename "${file}" .rs)"
        run_check "5" "p5-${target}" "Swarm scenario ${target}" \
            "cargo test -p bcinr-powl --test '${target}' -- --nocapture"
    done
fi

# Phase 6 — External FFI Contract
run_check "6" "p6-structure" "FFI contract and version admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 6"
run_check "6" "p6-tests" "FFI conformance" \
    "cargo test -p bcinr-ffi --test ffi_conformance -- --nocapture"
run_check "6" "p6-native" "FFI native build" \
    "cargo check -p bcinr-ffi --locked"
run_check "6" "p6-wasm" "FFI wasm32 build" \
    "cargo check -p bcinr-ffi --target wasm32-unknown-unknown --locked"

# Phase 7a — Hostile Mutants 6–11
run_check "7a" "p7a-structure" "Performance closure source admission" \
    "python3 scripts/verify_v26_7_26_structure.py --phase 7"
for mutant in 6 7 8 9 10 11; do
    run_check "7a" "p7a-mutant-${mutant}" "Isolated hostile mutant ${mutant}" \
        "cargo test -p bcinr-cmca --features 'mutant_${mutant}' --test hostile_mutants -- --nocapture"
done

# Phase 7b — Chaos Harness
run_check "7b" "p7b-chaos" "Chaos scenarios" \
    "cargo test -p bcinr-powl --test chaos_scenarios -- --nocapture"

# Phase 7c — Benchmarks
run_check "7c" "p7c-temporal-bench" "Temporal core benchmark" \
    "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo bench -p bcinr-pddl --bench phase1_temporal -- --noplot"
run_check "7c" "p7c-scheduler-bench" "Temporal scheduler benchmark" \
    "CARGO_PROFILE_BENCH_CODEGEN_UNITS=1 cargo bench -p bcinr-powl --bench phase3_scheduler -- --noplot"

# Phase 7d — Release hygiene and report integrity
run_check "7d" "p7d-format" "Formatting" \
    "cargo fmt --all -- --check"
run_check "7d" "p7d-clippy" "Release crates clippy" \
    "cargo clippy -p bcinr-pddl -p bcinr-powl -p bcinr-ffi --all-targets -- -D warnings"
run_check "7d" "p7d-lock" "Locked dependency graph" \
    "cargo metadata --locked --format-version 1 --no-deps >/dev/null"
run_check "7d" "p7d-tree" "Tracked tree remains unchanged" \
    "git diff --exit-code -- ."

PHASE_KEYS=("1" "2" "3" "4" "5" "6" "7a" "7b" "7c" "7d")
PHASE_NAMES=(
    "Temporal Core"
    "Resource Intervals"
    "Temporal Scheduler"
    "Evidence / OCEL"
    "Swarm Scenarios"
    "External FFI"
    "Mutants 6-11"
    "Chaos Harness"
    "Benchmarks"
    "Report and Release Hygiene"
)

FINISHED_AT="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
{
    printf '# BCINR v26.7.26 Verification Report\n\n'
    printf -- '- **Run ID:** `%s`\n' "${RUN_ID}"
    printf -- '- **Started:** `%s`\n' "${STARTED_AT}"
    printf -- '- **Finished:** `%s`\n' "${FINISHED_AT}"
    printf -- '- **Commit:** `%s`\n' "${HEAD_SHA}"
    printf -- '- **Initial tree state:** `%s`\n' "${TREE_STATE}"
    printf -- '- **Rust:** `%s`\n' "$(rustc --version 2>/dev/null || printf 'UNAVAILABLE')"
    printf -- '- **Cargo:** `%s`\n\n' "$(cargo --version 2>/dev/null || printf 'UNAVAILABLE')"
    printf '## Phase Status\n\n'
    printf '| Phase | Status |\n|---|---|\n'
    blocked=0
    alive=0
    for index in "${!PHASE_KEYS[@]}"; do
        key="${PHASE_KEYS[$index]}"
        status="$(phase_status "${key}")"
        printf '| %s. %s | **%s** |\n' "${key}" "${PHASE_NAMES[$index]}" "${status}"
        if [[ "${status}" == "ALIVE" ]]; then
            alive=$((alive + 1))
        else
            blocked=$((blocked + 1))
        fi
    done
    printf '\n## Executed Checks\n\n'
    printf '| Phase | Check | Exit | Seconds | Tests | Log | Command |\n'
    printf '|---|---|---:|---:|---|---|---|\n'
    python3 - "${LEDGER}" <<'PY'
import csv
import sys
from pathlib import Path
for row in csv.DictReader(Path(sys.argv[1]).open(), delimiter="\t"):
    command = row["command"].replace("|", "\\|")
    label = row["label"].replace("|", "\\|")
    summary = row["test_summary"].replace("|", "\\|")
    log = row["log"].replace("|", "\\|")
    print(f'| {row["phase"]} | {label} | {row["exit_code"]} | {row["duration_seconds"]} | {summary} | `{log}` | `{command}` |')
PY
    printf '\n## Receipt\n\n'
    printf 'This report is derived from `%s`; every row maps to a retained command log.\n\n' "${LEDGER#${REPO_ROOT}/}"
    if [[ "${blocked}" -eq 0 ]]; then
        printf '**BCINR v26.7.26: %s/10 release rails ALIVE. Ready for release.**\n' "${alive}"
    else
        printf '**BCINR v26.7.26: %s/10 release rails ALIVE; %s BLOCKED/UNKNOWN. Cannot ship.**\n' "${alive}" "${blocked}"
    fi
} > "${REPORT}"

# Hash the actual evidence after the report exists. SHA-256 is used only as an
# artifact transport checksum; semantic standing comes from the command ledger.
(
    cd "${EVIDENCE_ROOT}" || exit 1
    find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS
)

printf 'Report: %s\n' "${REPORT}"
printf 'Evidence: %s\n' "${EVIDENCE_ROOT}"
if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    printf 'report=%s\n' "${REPORT}" >> "${GITHUB_OUTPUT}"
    printf 'evidence_root=%s\n' "${EVIDENCE_ROOT}" >> "${GITHUB_OUTPUT}"
fi

for key in "${PHASE_KEYS[@]}"; do
    if [[ "$(phase_status "${key}")" != "ALIVE" ]]; then
        exit 1
    fi
done
exit 0
