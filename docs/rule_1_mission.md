# Rule 1: Mission

## Fundamental Mandate
BCINR is a deterministic computational substrate for bounded, branchless, allocation-free execution.

The authoritative runtime must preserve:
$$ \text{admitted input} \rightarrow \text{fixed instruction shape} \rightarrow \text{deterministic output} $$

## The 7 Criteria for Completeness
The repository does not accept implementations that merely appear correct in tests. Every authoritative primitive must have:

1. a mathematical contract;
2. a structurally lawful implementation;
3. an independent oracle or proof;
4. hostile mutants;
5. source-level verification;
6. object-code verification;
7. reproducible evidence.

A feature is not complete until all seven exist.
