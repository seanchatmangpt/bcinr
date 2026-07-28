#!/usr/bin/env python3
"""Compare bcinr against the POWL reference over the shared corpus.

Compares VERDICTS ("converted" / "refused"), not model structures. The two
implementations use different representations, and a translator between them
would be a third piece of code that could itself be the thing that is wrong.
The verdict is the one judgement both make in the same vocabulary.

Exit 1 on any disagreement. A disagreement is a finding in one of the two
implementations, not a tolerance to widen.
"""
import json, os, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))


def run(cmd, cwd=None):
    proc = subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)
    if proc.returncode != 0:
        sys.exit(f"{' '.join(cmd)} failed:\n{proc.stderr.strip()}")
    return [json.loads(line) for line in proc.stdout.splitlines() if line.strip()]


def main():
    if not os.path.isdir(os.path.join(HERE, "vendor", "powl")):
        sys.exit("oracle absent: run ./oracles/fetch.sh first")

    reference = {r["name"]: r for r in run([sys.executable, "powl/oracle.py"], cwd=HERE)}
    ours = {r["name"]: r for r in run(["cargo", "run", "-q"], cwd=os.path.join(HERE, "powl/runner"))}

    missing = set(reference) ^ set(ours)
    if missing:
        sys.exit(f"corpus mismatch, the two sides did not run the same cases: {sorted(missing)}")

    diverged = 0
    for name in reference:
        ref, our = reference[name]["verdict"], ours[name]["verdict"]
        if ref == our:
            print(f"  agree    {name:24} {ref}")
        else:
            diverged += 1
            print(f"  DIVERGE  {name:24} reference={ref} bcinr={our}")
            print(f"           reference: {reference[name]['detail']}")
            print(f"           bcinr:     {ours[name]['detail']}")

    print(f"\n{len(reference) - diverged}/{len(reference)} agree")
    return 1 if diverged else 0


if __name__ == "__main__":
    sys.exit(main())
