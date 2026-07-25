# Research Report: The Role of `@turing_machine`

Based on Rule 4 in the `AGENTS.md` constitution, here is the documented role and specific authority of `@turing_machine` within the BCINR Deterministic Substrate.

## Role Definition

`@turing_machine` acts as the **Enforcer of Determinism**. Its primary function is to serve as the structural auditor and merge gatekeeper for the project.

## Exclusive Authority

The `@turing_machine` holds exclusive authority over several critical enforcement policies:

*   **Cyclomatic-Complexity Enforcement:** Ensuring all authoritative code maintains strict $CC=1$.
*   **Authoritative-Call-Graph Classification:** Mapping and verifying the complete execution graph.
*   **Cheat-Scanner Policy:** Administering the rules that prevent scanner evasion and verification theater.
*   **Source Audit:** Verifying compliance at the AST/source level.
*   **Object-Code Audit:** Verifying compliance in the final release binary disassembly.
*   **Panic-Path Audit:** Ensuring no panic symbols are reachable.
*   **Allocation Audit:** Ensuring zero heap allocations (`no alloc`).
*   **Gate-Jurisdiction Audit:** Confirming that the authoritative crate and all its features fall within the correct verification jurisdictions.

## Required Actions & Enforcements

The Enforcer is mandated to perform and verify the following actions:

1.  **Strict Cyclomatic Complexity ($CC=1$)**: Verify that every authoritative function has a cyclomatic complexity of exactly 1.
2.  **Exhaustive Scanning**: Ensure that all private functions, macro expansions, generated Rust code, and build-script outputs are thoroughly scanned.
3.  **Jurisdictional Coverage**: Verify that the authoritative crate falls inside every relevant gate's jurisdiction.
4.  **Symbol Elimination**: Ensure that absolutely no panic symbols or allocator symbols are reachable in the authoritative path.
5.  **Branch and Loop Eradication**: Confirm that no unexpected branch instructions or runtime loop backedges exist in the compiled object code.
6.  **Arithmetic Safety**: Ensure no floating-point or division instructions exist unless explicitly admitted by policy.

## Governing Standard

The standard governing `@turing_machine` is encapsulated in the following law:

> **The authoritative instruction shape must not depend on semantic input.**

Furthermore, source-level claims are insufficient; the `@turing_machine` strictly requires **disassembly evidence** to substantiate structural compliance.
