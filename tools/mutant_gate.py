#!/usr/bin/env python3
"""Mutant kill gate — AGENTS.md §19 semantics.

For each injected mutant (cargo feature mutant_N of bcinr-cmca):
  1. the mutated crate must compile through the real build path
     (--no-run); a non-compiling mutant is CHEAT-009 theater, not a kill;
  2. the normal suite must FAIL under the mutant — that failure IS the kill.
A mutant whose suite passes SURVIVED and fails the gate, blocking feature
work until the hostile suite is strengthened to detect it.
"""
import subprocess
import sys

CRATE = ["-p", "bcinr-cmca"]
MUTANTS = [f"mutant_{i}" for i in range(1, 12)]


def run(args: list[str]) -> subprocess.CompletedProcess:
    return subprocess.run(args, capture_output=True, text=True)


def main() -> int:
    failures: list[tuple[str, str, str]] = []
    for mutant in MUTANTS:
        features = f"--features={mutant},std"
        build = run(["cargo", "test", *CRATE, features, "--no-run", "--quiet"])
        if build.returncode != 0:
            tail = (build.stderr or build.stdout).strip().splitlines()
            detail = tail[-1] if tail else "no diagnostics captured"
            failures.append((mutant, "BUILD_BROKEN", detail))
            print(f"{mutant}: BUILD_BROKEN — {detail}")
            continue
        run_out = run(["cargo", "test", *CRATE, features, "--quiet"])
        if run_out.returncode == 0:
            failures.append((mutant, "SURVIVED", "suite passed under the mutant"))
            print(f"{mutant}: SURVIVED — suite passed under the mutant")
        else:
            combined = (run_out.stdout or "") + (run_out.stderr or "")
            killed = sum(
                1
                for line in combined.splitlines()
                if "... FAILED" in line or line.strip().startswith("failures:")
            )
            print(f"{mutant}: KILLED (suite exit {run_out.returncode})")
    if failures:
        print()
        for mutant, kind, detail in failures:
            print(f"MUTATION_GATE_FAILED: {mutant} {kind} ({detail})")
        print(
            f"{len(failures)}/{len(MUTANTS)} mutants not properly killed — "
            "standing MUTATION_GATE_FAILED; feature work blocked (AGENTS.md §19)."
        )
        return 1
    print(f"\nMUTATION_GATE: all {len(MUTANTS)} mutants compiled and were killed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
