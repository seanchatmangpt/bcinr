---
name: evidence-report
description: Use when assembling the mandatory evidence artifact set (AGENTS.md §29) and the final completion report (AGENTS.md §31) for a bcinr feature — checks that every artifact exists and that no banned overclaiming phrase appears unqualified. Triggers on "write the final report", "evidence artifacts", "standing report", or before marking a bcinr feature complete.
---

# Evidence report

Implements AGENTS.md §29 (mandatory evidence artifacts) and §31 (required final report format).
No agent — including this one — self-certifies; this skill only checks that the required
artifacts exist and that claims are bounded, not that the underlying work is correct.

## Steps

1. **Confirm the required artifact set exists** for the feature:
   `CONTRACT.md`, `HOARE_TRIPLES.md`, `AUTHORITATIVE_CALL_GRAPH.md`, `SOURCE_AUDIT.md`,
   `OBJECT_CODE_AUDIT.md`, `ORACLE_INDEPENDENCE.md`, `MUTANT_KILL_MATRIX.md`,
   `NUMERIC_ERROR_REPORT.md`, `GATE_JURISDICTION.md`, `COMMAND_TRANSCRIPT.md`,
   `CURRENT_STATUS.md`. Where applicable also confirm `STABILITY_CERTIFICATE.md`,
   `CERTIFICATE_DIGEST.txt`, `GENERATED_DRIFT_REPORT.md`, `RECEIPT_REPLAY_REPORT.md`.
2. **Cross-check ownership** — contracts/oracle came from `@hoare-oracle`, audits from
   `@turing-machine`, mutant matrix from `@armstrong-fault`, implementation notes from
   `@von-neumann-bypass`. A report is not credible if one role authored artifacts reserved for
   another (§26-27).
3. **Assemble the final report** in the exact §31 shape:

   ```text
   1. Exact files changed
   2. Authoritative roots affected
   3. Mathematical contracts added
   4. Refusal variants added
   5. Independent oracle description
   6. Mutants injected
   7. Mutants killed
   8. Commands executed
   9. Gate jurisdiction
   10. Source CC results
   11. Allocation and panic audit
   12. Disassembly results
   13. Generated-code reproducibility
   14. Remaining unknowns
   15. Final standing
   ```

4. **Scan the draft report** for banned unqualified phrases: "looks correct," "should be
   branchless," "likely optimized," "appears safe," "all good," "production ready,"
   "mathematically proven." Each occurrence must be replaced with a specific bounded claim and
   linked evidence, or removed.
5. **Assign final standing** using only the bounded vocabulary in AGENTS.md §28 — never a label
   stronger than the weakest load-bearing dependency (e.g. don't report `BRANCHLESS_ALIVE` if the
   object-code audit only covered one target).

## Output

The completed 15-point report plus a pointer to each backing artifact file.

## Boundaries

- An artifact that is missing means the corresponding report line is `UNKNOWN`, not omitted.
- Do not let five agents repeating the same claim substitute for one linked artifact (§27).
