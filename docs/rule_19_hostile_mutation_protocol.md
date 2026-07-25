I have read `/Users/sac/bcinr/AGENTS.md` and extracted the details for the "Hostile mutation protocol" under Rule 19 as requested.

### 19. Hostile mutation protocol

For every implementation file, the exact 6-step protocol for injecting mutants and verifying typed refusals is as follows:

1. identify at least three load-bearing laws;
2. produce one mutant per law;
3. inject the mutant through the real build path;
4. run the normal suite;
5. verify the expected typed refusal or oracle mismatch;
6. record the kill evidence.

Additionally, the rule dictates that the mutant ledger must contain the following fields:

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

A surviving mutant changes project standing to `MUTATION_GATE_FAILED` and blocks all feature work.
