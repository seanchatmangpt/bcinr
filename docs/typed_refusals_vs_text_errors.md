# Typed Refusals vs. Text Errors in BCINR

According to the BCINR Deterministic Substrate Constitution (`AGENTS.md`), specifically **Rule 18 (Typed refusals)**, all rejected authoritative operations must produce a bounded typed refusal code. Human-readable text is strictly banished from the hot path.

## Why Human-Readable Text Errors Are Banned

The prohibition of string-based error messages is a direct consequence of BCINR's core architectural laws:

1. **Zero Allocation and Fixed Bounded Memory**: The authoritative hot path must operate under `#![no_std]` with absolute zero heap allocation. Formatting and storing human-readable strings inherently require dynamic memory allocation or variable-length buffers, which violate the `fixed bounded memory access` and `fixed-width outputs` laws.
2. **The Radon Law (CC=1) and Branchlessness**: Constructing error strings typically involves conditional formatting, iteration over buffers, or dynamic dispatch. This introduces data-dependent branches. In BCINR, control flow must remain `CC=1`, resolving entirely through branchless bit-parallel mask selection. 
3. **No Panic Paths**: Standard string formatting machinery in Rust carries hidden panic paths. `AGENTS.md` strictly prohibits any panic or unwinding paths in the transitive call graph.
4. **Execution Boundedness**: The substrate mandates `fixed bounded execution work`. String manipulation introduces variable execution time depending on the string length and formatting logic, creating potential timing side-channels.

To maintain a deterministic, branchless instruction shape, all failures must be represented as fixed-width types (like enums) that can be propagated and selected using bitwise masking. A rejected operation must leave persistent state bit-for-bit unchanged without returning plausible defaults or mutating partial state.

## Semantic Differences Between Required Refusal Categories

Typed refusals provide exact, machine-readable reasons for rejecting an operation without compromising the structural laws. 

### 1. `ContractViolation`
* **Domain**: Mathematical invariants and function preconditions.
* **Semantics**: Triggered when an operation breaches the strict Hoare logic contracts defined by the `@hoare_oracle`. This indicates that the input domain, conservation law, monotonicity law, or overflow behavior specified for a primitive was violated. It is a fundamental axiomatic failure where the input is mathematically inadmissible for the requested operation.

### 2. `ContractionMarginInsufficient`
* **Domain**: Autonomic control theory and stability verification.
* **Semantics**: Relates to **Rule 12 (No runtime theorem discovery)**. In BCINR, the runtime cannot dynamically search for stability; it must verify a supplied static domination witness. This refusal is emitted when the provided witness or adaptive mutation does not guarantee a sufficient contraction margin ($\delta$). It specifically rejects state changes that cannot mathematically prove they will safely converge.

### 3. `ObjectCodeAuditFailed`
* **Domain**: Structural compliance and certificate admission.
* **Semantics**: Relates to **Rule 20 (Object-code audit)** and the `ReceiptSound` law. Before adaptive mutation or execution is admitted, it must present valid certificates. This refusal is triggered if an operation or payload is rejected because it lacks proof that it has passed the rigorous structural disassembly audit by `@turing_machine` (which proves $CC=1$, zero allocation, and no hidden branches). It is a structural/administrative refusal, ensuring no unverified code execution is admitted.
