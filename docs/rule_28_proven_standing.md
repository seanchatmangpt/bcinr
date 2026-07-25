Based on `AGENTS.md` in the `bcinr` repository, here is the breakdown of the `PROVEN` standing:

### What `PROVEN` Means

In the context of the BCINR Deterministic Substrate Constitution (Rule 28: Standing vocabulary), the `PROVEN` standing is defined as:

> **"A specific theorem is machine-checked or exhaustively established over its declared domain."**

It represents a rigorous tier of mathematical correctness and assurance within the repository. It signifies that the logic or theorem in question isn't just empirically tested to be "alive", but structurally and formally verified to hold true for every possible valid input.

### Conditions for the `PROVEN` Label

For a piece of code (or a specific mathematical law governing that code) to achieve `PROVEN` standing, it must meet the strict strictures outlined under the `@hoare_oracle` jurisdiction (Rule 4) and achieve **Full-domain standing**. 

Because brute-force enumeration of an entire domain (like $2^{64}$) is often unfeasible, the constitution dictates that **random testing alone is never enough** to establish universal standing. 

Instead, to be labeled `PROVEN`, the implementation must be backed by one of the following authoritative artifacts:
1. **A formal proof**
2. **An exhaustive proof** over a finite partition whose cases mathematically cover the entire domain
3. **A bit-vector solver certificate** (such as from a SAT/SMT solver)
4. **An equivalent bounded theorem artifact**

Furthermore, as a prerequisite for this level of verification, the code must have a formalized Hoare contract explicitly detailing its preconditions, postconditions, valid input domain, output range, and conservation/monotonicity laws.
