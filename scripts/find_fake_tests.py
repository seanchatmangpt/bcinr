#!/usr/bin/env python3
"""Scan Rust test files for vacuous/mock/fake/stub test smells.

Heuristics, not proof: every hit needs a human (or agent) read to confirm it's
actually vacuous before "fixing" it. False positives are expected (e.g. a
comment that says "no mock is used").

Usage:
    python3 scripts/find_fake_tests.py [root...]
"""
import re
import sys
from pathlib import Path

# Phrases that are almost always a tell that an assertion was never written,
# only described in prose ("In a real system, X would happen" instead of
# actually happening).
NARRATION_TELLS = [
    r"\bIn a real\b",
    r"\bIn real\b",
    r"\bIn practice\b",
    r"\bIn production\b",
    r"\bIn full implementation\b",
    r"\bIn real implementation\b",
    r"\breal implementation\b",
    r"\bwould (?:be|need|require|trigger|cause|return)\b",
    r"\bplaceholder\b",
    r"\bPlaceholder\b",
    r"# TODO\b|// TODO\b",
    r"\bnot yet implemented\b",
]

# Structural smells detected line-by-line within a #[test] fn body.
STRUCTURAL_SMELLS = {
    "assert_true": re.compile(r"assert!\(\s*true\s*[,)]"),
    "trivial_self_eq": re.compile(r"assert_eq!\(\s*(\d+)\s*,\s*\1\s*[,)]"),
    "vacuous_if_let_ok": re.compile(r"if let Ok\(.*\)\s*=\s*result\s*\{"),
    "vacuous_len_gate": re.compile(r"if results\.len\(\) == \d+\s*\{"),
    "ignored_test": re.compile(r"#\[ignore\]"),
}

TEST_FN_RE = re.compile(r"#\[test\]\s*\n\s*(?:async\s+)?fn\s+(\w+)\s*\([^)]*\)\s*(?:->[^\{]+)?\{", re.MULTILINE)


def find_test_fn_bodies(text: str):
    """Yield (name, start_line, body_text) for each #[test] fn in a naive brace-matched way."""
    for m in TEST_FN_RE.finditer(text):
        name = m.group(1)
        brace_start = m.end() - 1  # the '{' just matched
        depth = 0
        i = brace_start
        for i in range(brace_start, len(text)):
            if text[i] == "{":
                depth += 1
            elif text[i] == "}":
                depth -= 1
                if depth == 0:
                    break
        body = text[brace_start : i + 1]
        start_line = text.count("\n", 0, m.start()) + 1
        yield name, start_line, body


def scan_file(path: Path):
    findings = []
    try:
        text = path.read_text(errors="replace")
    except OSError:
        return findings

    for name, line_no, body in find_test_fn_bodies(text):
        stripped = body.strip()
        if stripped in ("{}", "{\n}"):
            findings.append((line_no, "empty_test_body", name))
            continue

        has_assert = bool(re.search(r"\bassert(?:_eq|_ne)?!\(|\.unwrap\(\)|\.expect\(", body))
        if not has_assert:
            findings.append((line_no, "no_assertion_or_unwrap", name))

        for smell_name, pat in STRUCTURAL_SMELLS.items():
            if pat.search(body):
                findings.append((line_no, smell_name, name))

        for tell in NARRATION_TELLS:
            m = re.search(tell, body)
            if m:
                snippet_line = body.count("\n", 0, m.start())
                findings.append((line_no + snippet_line, f"narration_tell:{tell}", name))

    return findings


def main(argv):
    roots = [Path(a) for a in argv] or [Path(".")]
    total_hits = 0
    for root in roots:
        for path in sorted(root.rglob("*.rs")):
            if "target" in path.parts:
                continue
            if "/tests/" not in f"/{path}" and not path.name.endswith("_test.rs") and "tests" not in path.parts:
                # Only scan test files / anything under a tests/ dir.
                if not any(part == "tests" for part in path.parts):
                    continue
            findings = scan_file(path)
            if findings:
                print(f"\n{path}")
                for line_no, kind, fn_name in findings:
                    print(f"  L{line_no:<5} {kind:<28} {fn_name}")
                total_hits += len(findings)
    print(f"\n{total_hits} potential smell(s) found.")


if __name__ == "__main__":
    main(sys.argv[1:])
