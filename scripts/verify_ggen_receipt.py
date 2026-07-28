#!/usr/bin/env python3
"""Classify a fresh ggen manufacturing receipt without granting release standing.

`ggen sync` runs before compilation, tests, and evidence verification. Unknown
post-manufacture dimensions are therefore expected at this stage. This wrapper
admits the manufacturing act only when every generated output was observed and
config standing is equivalent. It explicitly overrides any premature producer
claim that the artifact is promotion-eligible; the later closure rails alone can
upgrade the complete repository to release standing.
"""

import json
import sys
from pathlib import Path


def state(value):
    if isinstance(value, str):
        return value
    if isinstance(value, dict) and len(value) == 1:
        return next(iter(value))
    return "Malformed"


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else ".ggen-v2/receipt.json")
    try:
        record = json.loads(path.read_text(encoding="utf-8"))["record"]
        v2 = record["v2"]
        equivalence = v2["equivalence"]
        admissions = v2["admission"]["Recorded"]
    except (OSError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"GGEN_MANUFACTURE_REFUSED: {error}")
        return 1

    failures = []
    if record.get("andon") != "Green":
        failures.append(f"andon={record.get('andon')}")
    if not admissions:
        failures.append("no generated output admissions")
    for admission in admissions:
        if admission.get("decision") != "Admitted" or admission.get("observed_outcome") != "Pass":
            failures.append(
                f"{admission.get('evidence_id', '<unknown>')}: "
                f"decision={admission.get('decision')} "
                f"outcome={admission.get('observed_outcome')}"
            )

    config_state = state(equivalence.get("config"))
    if config_state != "Equivalent":
        failures.append(f"config={config_state}")
    source_state = state(equivalence.get("source"))
    if source_state in {"Unknown", "Malformed"}:
        failures.append(f"source={source_state}")

    if failures:
        print("GGEN_MANUFACTURE_REFUSED")
        for failure in failures:
            print(f"- {failure}")
        return 1

    unresolved = {
        key: state(equivalence.get(key))
        for key in ("compiled_binary", "docs", "tests", "receipts", "evidence", "gates")
        if state(equivalence.get(key)) not in {"Equivalent", "Verified", "Pass", "Green"}
    }
    if v2.get("promotion_eligible") and unresolved:
        print("GGEN_PREMATURE_PROMOTION_OVERRIDDEN")
        for key, value in unresolved.items():
            print(f"- {key}={value}")

    print("GGEN_MANUFACTURE_ADMITTED_PARTIAL")
    print("- release standing remains unavailable until the closure ladder completes")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
