Here are the details regarding the `CONTRACT.md` and `SOURCE_AUDIT.md` mechanical artifacts, as defined by Rule 29, the constitutional checkpoints in `AGENTS.md`, and their respective agent roles:

### `CONTRACT.md`
**Owner:** `@hoare_oracle` (Axiomatic proof lead and specification owner)
**Associated Checkpoint:** Checkpoint 1 — Contract

**What it contains:**
A formal mathematical contract for every authoritative primitive, modeled as a Hoare contract (`{P(x)} f(x) {Q(x,f(x))}`). It specifically defines:
- Preconditions, postconditions, invariants, and algebraic laws.
- The valid input domain and output range.
- Conservation laws and monotonicity laws (where applicable).
- Overflow behavior and the numeric error envelope.
- Invalid-input refusal conditions (strict typed refusals).
- Determinism and precise state-mutation boundaries.

**Why it is required:**
It serves as the foundation for BCINR's "Contract with Teeth." The constitution dictates that no implementation may begin until these domains, laws, and boundaries are mathematically fixed. It separates the specification from the implementation, guaranteeing that if an implementation deviates from this axiomatic reference by even 1 bit, verification will fail. Any implementation claim made outside of this artifact is considered to have "no standing."

---

### `SOURCE_AUDIT.md`
**Owner:** `@turing_machine` (Structural auditor and merge gatekeeper)
**Associated Checkpoint:** Checkpoint 5 — Source gates

**What it contains:**
A comprehensive structural audit of the authoritative source code, providing evidence of:
- **`CC=1` Enforcement (The Radon Law):** Proof that every authoritative function has a cyclomatic complexity of exactly 1.
- **AST & Macro Scanning:** Evidence that all private functions, trait methods, macro expansions, and generated Rust code have been scanned for hidden branches.
- **Allocation & Panic Path Analysis:** Proof of zero heap allocations (adhering to the Zero-Allocation Boundary) and the total absence of reachable panic paths.
- **Cheat Scanner Results:** Verification that the code does not violate anti-cheat laws (e.g., hiding branches in abstractions, using self-canceling operations, scanner evasion).
- **Gate Jurisdiction:** Proof that the changed files actually fall within the jurisdiction of the scanners being run.

**Why it is required:**
It enforces the deterministic computational limits of the repository at the source level. Since developers cannot simply claim "the function contains no `if`," this artifact systematically proves that the rules of branchless, allocation-free execution have not been bypassed via macros, private helpers, or dependencies. Without this exact mechanical artifact, the source code cannot advance to the object-code verification phase.
