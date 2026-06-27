# ADR 0001: Education Mode Lifecycle

## Status: ADMITTED

## Context

Sean needs a systematic, receipt-gated workflow for weekly education output.
Manual tracking leads to inconsistency and no audit trail.

## Decision

Implement education-mode as a PDDL8 domain with 5 parallel lanes.
Each publish action is receipt-gated (BLAKE3).
The `publish_education_week` action has exactly 8 preconditions (at the Need9 boundary).

## Consequences

- All lane completions are machine-verifiable
- OCEL trace available for process mining
- Education week admission requires explicit `executeTape` command
- No hand-flip path: receipt is the only admission gate
