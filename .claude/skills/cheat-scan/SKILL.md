---
name: cheat-scan
description: Use to run bcinr's cheat-scanner gate (AGENTS.md §16-17) over changed files before reporting a scan as green — parses the AST, macro expansions, and generated output, and reports findings as CHEAT[rule-id]. Triggers on "scan for cheats", "run the cheat scanner", "CHEAT-0xx", or before any merge report.
---

# Cheat scan

Implements AGENTS.md §16 (anti-cheat manifesto) and §17 (cheat-scanner requirements). Owned by
`@turing-machine`.

## Steps

1. **Determine jurisdiction first.** List every changed file, its features, and its target(s).
   Confirm the scanner command you are about to run actually covers all of them — a green run
   whose jurisdiction excludes a changed file is CHEAT-010 (gate-jurisdiction theater), not
   evidence.
2. **Run the scanner** across source, tests, benches, and generated output:
   `cargo make scan-cheats` (or the project's admitted equivalent of `bcinr-cheat-scanner`).
3. **Confirm scan coverage** includes: public and private functions, macro definitions and
   *expanded* output, generated Rust, whitespace-normalized and numeric-separator-stripped
   comparisons, equivalent hex spellings (e.g. `0xDEADBEEF` vs `0xDEAD_BEEF`), test references, and
   benchmark targets.
4. **Classify every finding** against the CHEAT-001..CHEAT-010 catalogue in §16:
   - CHEAT-001 self-canceling ops, CHEAT-002 circular oracle, CHEAT-003 magic constants,
     CHEAT-004 artificial file inflation, CHEAT-005 boilerplate verification claims,
     CHEAT-006 scanner evasion, CHEAT-007 dead-path compliance, CHEAT-008 benchmark theater,
     CHEAT-009 mutant theater, CHEAT-010 gate-jurisdiction theater.
5. **Report each finding** as `CHEAT[rule-id]: <exact file, span, description>`. Every finding
   blocks merge; no baseline suppression without a separately admitted waiver artifact.

## Output

A findings list in the `CHEAT[rule-id]` format (empty list only if jurisdiction was proven
complete and the scan actually ran clean — not merely "not run").

## Boundaries

- Do not report a scan as passing without stating which files/features/targets were inspected.
- Do not silently add a baseline suppression to make a finding disappear.
