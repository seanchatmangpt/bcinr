# How to Gate a Commit with the Cheat-Scanner and Contract-Gate

**Goal:** Catch hidden branches, forbidden operators, and fake-proof boilerplate *before* you commit, using the project's two custom static analyzers.

**Prerequisites:** A checkout that compiles. The two tools live under [`tools/`](../../../tools/) and are wired into [`Makefile.toml`](../../../Makefile.toml) as `scan-cheats` and `contract-gate`. Both scan only `crates/bcinr-logic/src/algorithms/`.

## Steps

1. Run the contract-gate. It parses every algorithm file with `syn` and enforces two invariants on each public function: cyclomatic complexity must be exactly 1 (no `if`/`match`/`for`/`while`/`loop`), and the function (or its file) must carry a `# Branchless Contract` doc section:

   ```bash
   cargo make contract-gate
   ```

   A failure looks like:

   ```text
   FAIL: my_kernel in crates/bcinr-logic/src/algorithms/my_kernel.rs has Cyclomatic Complexity 2 (Branch detected!)
   MISSING_U64_CONTRACT: my_kernel in crates/bcinr-logic/src/algorithms/my_kernel.rs
   ```

2. Fix complexity failures by removing control flow — convert the branch to a mask-and-select (see [guide-2](./guide-2.md)). Fix a missing-contract failure by adding the doc marker to the function:

   ```rust
   /// my_kernel
   ///
   /// # Branchless Contract
   /// **Invariant:** Execution path is independent of input data values.
   pub fn my_kernel(val: u64, aux: u64) -> u64 { /* ... */ }
   ```

3. Run the cheat-scanner. It is a heuristic linter that flags five anti-patterns: self-canceling XOR (e.g. `x.wrapping_add(y) ^ x`), a `_reference` oracle whose body is identical to the implementation it claims to verify, magic constants (`0xDEADBEEF`/`0xCAFEBABE`) in production code, artificial padding blocks, and boilerplate "Hoare-logic" lines posing as proofs:

   ```bash
   cargo make scan-cheats
   ```

   A clean run prints `OK: no cheat patterns detected across N algorithm files.`

4. Run both as part of the full pre-commit pipeline, which also covers fmt, check, clippy, tests, and the security audits:

   ```bash
   cargo make ci   # fmt, check, clippy, scan-cheats, contract-gate, test, audit, deny
   ```

## Verify it worked

- `cargo make contract-gate` ends with `Verified N public primitives` and a nonzero process exit only on real failures.
- `cargo make scan-cheats` reports zero findings; any finding exits non-zero and names the file and line to fix.
- To prove the scanner actually bites, temporarily add `let _ = 0xDEADBEEFu32;` to an algorithm file outside a `#[cfg(test)]` block, re-run, and watch it report `CHEAT[MAGIC_CONST]` — then revert.

See also: [Verify a function compiles to branchless code](./guide-1.md), [Replace an if/else hot path with mask::select](./guide-2.md), [Anti-Patterns](../explanation/anti-patterns.md).
