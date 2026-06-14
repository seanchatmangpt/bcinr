#!/usr/bin/env python3
"""Deterministically strip PADDING and FAKE_PROOF boilerplate from algorithm files.

Removes two pure-comment cheat patterns flagged by bcinr-cheat-scanner:
  * FAKE_PROOF  — 25 identical "Hoare-logic Verification Line N:" comment lines.
  * PADDING     — the "PADDING ENSURING FILE LENGTH REQUIREMENT" block that runs
                  from its leading separator to EOF.

Both patterns are comments only; the function return expression always precedes
them, so removal cannot alter behavior. A safety guard refuses to truncate any
file that has real code after the padding marker. Idempotent and git-reversible.
"""

import re
import sys
from pathlib import Path

ALGO_DIR = Path("crates/bcinr-logic/src/algorithms")

FAKE_PROOF_RE = re.compile(r"^\s*//\s*Hoare-logic Verification Line \d+:")
PADDING_MARKER = "PADDING ENSURING FILE LENGTH REQUIREMENT"
SEPARATOR_RE = re.compile(r"^\s*//\s*-{3,}\s*$")


def is_comment_or_blank(line: str) -> bool:
    s = line.strip()
    return s == "" or s.startswith("//")


def strip_file(path: Path) -> tuple[bool, int]:
    """Return (changed, lines_removed)."""
    original = path.read_text().splitlines()
    lines = list(original)

    # 1. Drop FAKE_PROOF lines.
    lines = [ln for ln in lines if not FAKE_PROOF_RE.match(ln)]

    # 2. Truncate the PADDING block (separator-aware, safety-guarded).
    pad_idx = next((i for i, ln in enumerate(lines) if PADDING_MARKER in ln), None)
    if pad_idx is not None:
        # Safety: everything from the marker to EOF must be comment/blank.
        if all(is_comment_or_blank(ln) for ln in lines[pad_idx:]):
            start = pad_idx
            # Absorb the leading separator line directly above the marker.
            if start > 0 and SEPARATOR_RE.match(lines[start - 1]):
                start -= 1
            lines = lines[:start]
        else:
            print(f"  SKIP padding (code after marker): {path}", file=sys.stderr)

    # 3. Trim trailing blank lines.
    while lines and lines[-1].strip() == "":
        lines.pop()

    removed = len(original) - len(lines)
    if removed == 0:
        return False, 0

    path.write_text("\n".join(lines) + "\n")
    return True, removed


def main() -> int:
    if not ALGO_DIR.is_dir():
        print(f"error: {ALGO_DIR} not found (run from repo root)", file=sys.stderr)
        return 2

    changed_files = 0
    total_removed = 0
    for path in sorted(ALGO_DIR.glob("*.rs")):
        if path.name == "mod.rs":
            continue
        changed, removed = strip_file(path)
        if changed:
            changed_files += 1
            total_removed += removed
            print(f"  {path.name}: -{removed} lines")

    print(f"\nStripped {total_removed} lines from {changed_files} files.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
