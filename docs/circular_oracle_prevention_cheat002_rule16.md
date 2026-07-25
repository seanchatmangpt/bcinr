# Enforcement of Oracle Independence (CHEAT-002)

In the BCINR Deterministic Substrate, **Rule 16 (CHEAT-002)** defines a "Circular Oracle" as *a reference implementation copied from the production implementation.* 

This directly ties into **Rule 15 (Independent Oracle Law)**, which explicitly prohibits creating oracles that are merely typed clones of the production code. Specifically, the following are banned:
* Line-by-line translation of production code.
* **Identical control structures with `f64`** (swapping fixed-point integers for floats but keeping the logic identical).
* Reuse of production normalization, lookup tables, or fixed-point helpers.

### Why Identical Control Structures are Banned
If an oracle merely uses the exact same control flow as the production code but swaps out the arithmetic types (e.g. `u64` to `f64`), it blindly inherits any structural defects or algorithmic errors present in the production logic. The test suite would trivially pass because both algorithms make the same mistake. By banning this, BCINR ensures verification relies on a truly distinct derivation of mathematical truth (such as direct mathematical formulas, Hoare specifications, or SMT bit-vector models).

### Structural Prevention Mechanism

The project relies on a mechanical enforcement mechanism via the **`bcinr-cheat-scanner`** to detect and block these circular oracles during the CI gate process. 

The mechanism works as follows:

1. **AST Parsing**: The scanner uses the `syn` crate to parse each Rust source file into an Abstract Syntax Tree (AST), identifying all function and method declarations.
2. **Body Normalization**: For each function, the scanner extracts the function body block, converts it to a string representation using the `quote` crate, and systematically strips all whitespace (`quote::quote!(#block).to_string().replace(" ", "")`).
3. **Pairwise Analysis**: 
   - It searches for functions ending in `_reference` or `_oracle` (e.g., `compute_oracle`).
   - It strips that suffix to find the corresponding production implementation base name (e.g., `compute`).
   - It compares the normalized AST string representations of the two function bodies.
4. **Violation Trigger**: If the normalized bodies match exactly, the scanner throws a `CHEAT[CHEAT-002]: circular oracle` violation, bringing the Substrate Integrity Score (SIS) to 0 and blocking the merge.

Because this enforcement occurs at the AST level and strips whitespace, developers cannot evade the scanner through superficial formatting changes. The oracle must be genuinely structurally and logically distinct to pass the pipeline.
