# Bit-Vector Solver Certificates in BCINR

According to the `bcinr/AGENTS.md` constitution, specifically **Rule 4: Full-domain requirement** and **Rule 15: Independent oracle law**, bounded execution and mathematical invariants must be proven across the full $2^{64}$ input domain. Since random testing cannot establish universal standing, and brute-force execution of $2^{64}$ permutations is computationally impossible (requiring centuries on modern hardware), the `@hoare_oracle` requires a **bit-vector solver certificate** or an **equivalent bounded theorem artifact**.

This document explains the mathematical mechanics of how SAT/SMT (Satisfiability Modulo Theories) solvers achieve total domain coverage without brute-forcing, and what constitutes a valid "certificate."

---

## 1. The Impossibility of Brute Force vs. The Power of Symbolic Logic

A 64-bit function $f(x)$ has $18,446,744,073,709,551,616$ possible inputs. Even at one billion checks per second, exhaustive enumeration takes nearly 600 years. If a function takes two 64-bit integers $f(x, y)$, the space becomes $2^{128}$, which is physically impossible to traverse.

To prove an invariant—such as ensuring a branchless SWAR (SIMD within a register) clamp never overflows, or a fixed-point selection mask exactly mirrors a mathematical ideal—we do not evaluate inputs sequentially. Instead, we use **symbolic execution** and **bit-vector logic** within an SMT solver (such as Z3 or CVC5). 

We define the Hoare contract: $\{P(x)\} \quad f(x) \quad \{Q(x, f(x))\}$. 
To prove this holds universally, we ask the solver to find **any** input $x$ that satisfies the precondition $P(x)$ but **violates** the postcondition $Q$.
We query the solver for the satisfiability of: $P(x) \land \neg Q(x, f(x))$.

*   If the solver finds a valid assignment, that assignment is a **counterexample** (a bug).
*   If the solver mathematically proves that **no such assignment exists (UNSAT)**, the invariant is structurally proven for the entire $2^{64}$ domain.

## 2. Bit-Blasting: Translating Arithmetic to Boolean Logic

Bit-vector solvers do not treat a 64-bit integer as a single abstract number; they treat it as an array of 64 individual boolean variables (bits). The process of translating fixed-width operations into propositional logic is called **bit-blasting**.

1. **Variables:** A 64-bit input `x` becomes a vector $[x_0, x_1, ..., x_{63}]$.
2. **Operations:** A branchless operation like `a.wrapping_add(b)` is modeled exactly like a hardware adder circuit. It expands into a network of AND, OR, and XOR gates representing the carry-lookahead or ripple-carry logic for all 64 bits. Bitwise operations (like `&`, `|`, `^`, `>>`) translate directly to routing logic and simple gates.
3. **Equivalence Check:** The independent oracle (the mathematical truth) and the production logic (the $CC=1$ branchless implementation) are structurally compared.

Once the entire problem is bit-blasted, it becomes an enormous boolean formula in Conjunctive Normal Form (CNF), ready for a SAT solver.

## 3. How the SAT Solver Explores $2^{64}$ Without Brute-Forcing

Modern SAT solvers bypass the combinatorial explosion of $2^{64}$ using the **CDCL (Conflict-Driven Clause Learning)** algorithm:

*   **Boolean Constraint Propagation (BCP):** When the solver sets a single bit to 0 or 1, it immediately determines the necessary states of connected logic gates. This prunes billions of dead-end possibilities in microseconds.
*   **Conflict Learning:** If the solver encounters a contradiction (e.g., bits $x_3$ and $x_{15}$ cannot simultaneously be true without violating a rule), it dynamically generates a new mathematical rule (a "learned clause") that bans that entire subspace of inputs forever.
*   **Structural Reductions:** Instead of checking individual numbers, the solver eliminates entire classes of numbers structurally. It reasons about the topology of the logic, proving that certain output bit patterns are physically unreachable given the initial logic gates.

Through CDCL, the solver can traverse and eliminate the entire $2^{64}$ domain in milliseconds or seconds, without ever checking the numbers one-by-one.

## 4. The "Solver Certificate" (The Bounded Theorem Artifact)

When the solver concludes that no counterexample exists (the formula is UNSAT), its word alone is not enough for the `@hoare_oracle`. A verified runtime demands an **externally checkable proof**.

An SMT/SAT solver certificate (such as a **DRAT** or **LRAT** proof artifact) is a cryptographic-like log of every mathematical deduction the solver made. 

1. **The Proof Sequence:** It contains the exact sequence of resolution steps (combining known truths to deduce new truths) from the initial bit-blasted formula to a final, undeniable contradiction (the empty clause).
2. **Independent Verification:** The certificate is checked by a very small, heavily audited **Proof Checker** (such as `drat-trim` or a formally verified checker written in Coq/Lean). The checker runs in linear time.
3. **Artifact Standing:** Once the proof checker accepts the certificate, this static file becomes the **equivalent bounded theorem artifact**. It is checked into the repository alongside the implementation, proving unequivocally that the branchless $CC=1$ implementation correctly maps to the independent oracle across the entire input domain.

## 5. Compliance with the Substrate Constitution

By requiring a Bit-Vector Solver Certificate, the BCINR architecture enforces the following guarantees:

*   **Rule 4 (Full-Domain Requirement):** The SAT proof acts as the exhaustive proof over the $2^{64}$ domain, mathematically avoiding the impossible brute-force requirement.
*   **Rule 15 (Independent Oracle Law):** The bit-blasted model serves as a mathematically distinct truth reference against the production code.
*   **Absolute Runtime Laws:** It ensures branchless mechanics (`select(mask, a, b)`) genuinely conserve logic without producing hidden arithmetic overflows or state corruption.
