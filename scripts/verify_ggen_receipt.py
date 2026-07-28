#!/usr/bin/env python3
"""Reject a ggen promotion claim unless every equivalence dimension is observed."""
import json
import sys
from pathlib import Path

GOOD = {"Equivalent", "Verified", "Pass", "Green"}
REQUIRED = ("source", "compiled_binary", "docs", "tests", "receipts", "evidence", "gates", "config")


def state(value):
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and len(value) == 1:
        return next(iter(value))
    return "Malformed"


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else ".ggen-v2/receipt.json")
    try:
        record = json.loads(path.read_text())["record"]
        v2 = record["v2"]
        equivalence = v2["equivalence"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"GGEN_RECEIPT_REFUSED: {error}")
        return 1

    failures = [f"{key}={state(equivalence.get(key))}" for key in REQUIRED if state(equivalence.get(key)) not in GOOD]
    if record.get("andon") != "Green":
        failures.append(f"andon={record.get('andon')}")
    if v2.get("standing_ceiling") != "Green":
        failures.append(f"standing_ceiling={v2.get('standing_ceiling')}")
    if not v2.get("promotion_eligible"):
        failures.append("promotion_eligible=false")
    if failures:
        print("GGEN_RECEIPT_REFUSED")
        for failure in failures:
            print(f"- {failure}")
        return 1
    print("GGEN_RECEIPT_ADMITTED")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
