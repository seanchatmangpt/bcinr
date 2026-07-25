# CHEAT-002: Circular Oracles & Structural Independence

In the BCINR Deterministic Substrate, the system strictly enforces the **Independent Oracle Law** to guarantee the mathematical integrity of the runtime. A common developer pitfall—often done out of convenience—is to copy-paste the production implementation into the test suite, rename it to `_reference`, and use it as an "oracle" to verify the production code. This effectively proves nothing more than "the code does what the code does."

The system classifies this pattern as **CHEAT-002 (Circular Oracle)** and strictly prohibits it.

## 1. The Policy and Governance

According to the Constitution (`AGENTS.md`) and the Independence Protocol (`docs/independent_oracles.md`):

- **No Self-Certification**: The implementation owner (`@von_neumann_bypass`) is strictly prohibited from authoring the final oracle and self-certifying equivalence. The oracle must be reviewed by the axiomatic proof lead (`@hoare_oracle`).
- **Logically Distinct**: An oracle is not independent merely because it is located in a `tests/` directory. It must be structurally and logically distinct from the authoritative branchless code.
- **Prohibited Patterns**:
  - Line-by-line translation of production code.
  - Reuse of production normalization, lookup tables, or fixed-point helpers.
  - Identical control structure (even if ported to `f64`).
  - Importing the authoritative function and wrapping it.

### Valid Oracles
A mathematically valid independent oracle must take one of the following forms:
- SAT/SMT Bit-Vector Model
- Arbitrary-Precision Implementation (e.g., `BigInt` / `BigRational`)
- Abstract State Machine
- Direct Mathematical Formula
- Hoare Specification
- Symbolic Proof
- Exhaustive Reduced-Domain Enumerator

## 2. Automated Enforcement

Policy alone is insufficient; the BCINR architecture enforces compliance mechanically via the `bcinr-cheat-scanner`. The `CHEAT-002` rule operates at the Abstract Syntax Tree (AST) layer to detect structural plagiarism.

### How the Cheat Scanner Works (`tools/bcinr-cheat-scanner/src/main.rs`)

1. **AST Parsing**: The scanner parses the Rust source code using the `syn` crate, stepping through every function and method.
2. **Oracle Identification**: It identifies potential oracles by looking for functions with names ending in `_reference` or `_oracle`.
3. **Target Resolution**: It determines the base implementation name by stripping the suffix (e.g., `calculate_mass_oracle` targets `calculate_mass`).
4. **AST Normalization**: The scanner extracts the AST block for both the implementation and the oracle. It stringifies the tokens using `quote::quote!()` and strips all whitespace (`.replace(" ", "")`). This means formatting, comments, or indentation changes cannot be used to evade the scanner.
5. **Equivalence Checking**: If the normalized AST string of the oracle perfectly matches the AST string of the implementation, the system flags a violation:
   
   ```
   CHEAT[CHEAT-002]: <file_path> — circular oracle: <oracle_name> identical to implementation <base_name>
   ```

### The CI Consequence

If `CHEAT-002` is detected, the cheat scanner exits with a non-zero status code, immediately failing the `cargo make scan-cheats` gate. This completely blocks the PR from merging. A feature is never considered complete, and the Substrate Integrity Score (SIS) drops to 0, until a genuinely independent, structurally distinct mathematical oracle is provided.
