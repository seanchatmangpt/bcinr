# Hostile Mutation Protocol

According to Rule 19 of the BCINR Deterministic Substrate Constitution (`AGENTS.md`), the Hostile Mutation Protocol is a strict adversarial verification mechanism used to ensure test suite adequacy and absolute deterministic integrity.

## The 6-Step Process
For every implementation file, the following protocol must be strictly followed:
1. **Identify** at least three load-bearing laws.
2. **Produce** one mutant per law.
3. **Inject** the mutant through the real build path.
4. **Run** the normal suite.
5. **Verify** the expected typed refusal or oracle mismatch.
6. **Record** the kill evidence.

## The Mutant Ledger
The results of this process must be meticulously recorded. The mutant ledger must contain the following exact fields:
- mutant id
- source file
- changed law
- exact mutation
- expected detection
- actual detection
- test name
- receipt digest
- standing

## Surviving Mutants and Feature Freezes
A surviving mutant immediately changes the project standing to `MUTATION_GATE_FAILED` and strictly blocks all feature work. 

**Why it freezes feature work:** 
In the BCINR substrate, correctness is not assumed—it is mathematically and structurally enforced. A surviving mutant demonstrates that a core load-bearing law can be corrupted without the test suite detecting a typed refusal or oracle mismatch. As defined in Rule 4 (`@armstrong_fault`), *"A suite that cannot kill a plausible mutant is itself defective."* Permitting feature work to continue under these conditions would violate the absolute runtime laws of the substrate. Therefore, all development must halt until the verification failure is rooted out and the test suite's deterministic integrity is restored.
