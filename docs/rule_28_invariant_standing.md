Based on my review of `AGENTS.md`, here are the details regarding the `INVARIANT` standing from Rule 28:

### What `INVARIANT` Means
In Rule 28 ("Standing vocabulary"), `INVARIANT` is defined strictly as:
> **True by construction or type exclusion.**

### Conditions for the `INVARIANT` Standing
A piece of code or a specific property is labeled with the `INVARIANT` standing when its correctness is guaranteed at compile-time rather than verified at runtime. Specifically, it applies under the following conditions:

1. **Type Exclusion:** The type system is used to make invalid states unrepresentable. If a piece of data is of a certain type, the compiler guarantees that it adheres to the rules (e.g., using fixed-width or strongly typed wrappers that cannot express an out-of-bounds or unlawful value).
2. **True by Construction:** The structural design of the logic ensures the property is inherently true. The code is written in such a way that the sequence of operations or mathematical structures cannot yield a contract-violating result, regardless of the input.

In the context of the BCINR deterministic substrate, `INVARIANT` represents one of the strongest claims you can make. It means that the property does not merely rely on test coverage or external oracle proofs; it is physically impossible for the code to violate the property and still compile.
