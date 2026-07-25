# CHEAT-031: Black Box Branchlessness Claims

## Why is it forbidden to claim `core::hint::black_box` intrinsically guarantees branchlessness?
In `bcinr`, it is strictly forbidden to claim that `core::hint::black_box` (or any source-level construct) intrinsically guarantees machine-level branchlessness. This prohibition is enforced by the `bcinr-cheat-scanner` as **`CHEAT-031`**.

1. **LLVM Optimizations:** `core::hint::black_box` is merely an optimization barrier or hint to the compiler to prevent constant folding. However, LLVM backend optimization passes are still capable of rewriting bitwise or branchless logic back into conditional jumps (JCC instructions) if the compiler's heuristics determine it might be more optimal for the target architecture. 
2. **No Source Guarantees:** As stated in the scanner specifications, "No source construct can guarantee LLVM emits branchless object code."
3. **Constitutional Standard (Rules 3 & 7):** The BCINR Constitution explicitly dictates that a "branchless Rust function compiling into input-dependent jumps is a violation." Rule 7 expressly prohibits the claim: *"The function contains no `if`, therefore it is branchless."* Any documentation, code comment, or PR claiming that wrapping a variable in a black box guarantees no branches will be emitted is treated as false and flagged by the scanner.

## How the Constitution Mandates Object-Code Audits
Because no source-level construct guarantees that LLVM will emit branchless object code, the constitution mandates rigorous object-code audits to prove compliance.

1. **Rule 20 (Object-code audit):** Every supported release target requires an exact production-profile disassembly audit. 
2. **Disassembly Verification:** The audit must systematically inspect all authoritative root symbols, all transitive helper symbols, and compiler intrinsics. It must explicitly verify the exact count of conditional jumps and loop backedges in the final compiled machine code. 
3. **Source `CC=1` is Insufficient:** Achieving Cyclomatic Complexity of 1 (`CC=1`) at the source level using AST parsing is necessary but insufficient. The true arbiter of branchlessness is the final assembled object code.
4. **Permitted Claim:** The only constitutionally permitted claim regarding branchlessness is: *"The full authoritative call graph contains no input-dependent conditional branch in the audited release object code for the declared target."*

Through this mandate, `bcinr` ensures that determinism and constant-time execution are proven against the actual machine code rather than assumed from Rust source semantics.
