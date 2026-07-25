Based on my research of `AGENTS.md` under **Rule 19. Hostile mutation protocol**, here are the details you requested:

### Required Format of the Mutant Ledger
The mutant ledger must contain the following fields:

```text
mutant id
source file
changed law
exact mutation
expected detection
actual detection
test name
receipt digest
standing
```

### Consequences of a Surviving Mutant
If a mutant survives, the following actions occur:
1. The project standing is immediately changed to `MUTATION_GATE_FAILED`.
2. All feature work is blocked.
3. Additionally, per **Rule 24**, a surviving mutant is considered an "absolute failure." This forces the Substrate Integrity Score (SIS) to `0` and triggers the `MaturityScrutiny` protocol (which requires freezing development, quarantining code, repairing the defect, and rerunning all gates).
