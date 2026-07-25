### `ReceiptRejected` Definition
In the `bcinr` codebase, according to **Rule 18** of `AGENTS.md`, `ReceiptRejected` is a bounded typed refusal code triggered when a structural receipt fails internal validation (e.g., cryptographic digest mismatches, structural bound violations, or unsatisfied temporal window constraints). 

It acts as the deterministic enforcer of **Rule 11** (`ReceiptSound` Law), which mandates that adaptive state mutation strictly requires a valid set of accepted receipts.

*Note: While heavily documented in the `docs/` directory as a required category under the `AGENTS.md` constitution, the specific `ReceiptRejected` identifier does not currently appear in the active source code implementations of `bcinr-cmca` or `bcinr-api`.*

### Branchless Mathematical Trigger Condition
To comply with the **Radon Law ($CC=1$)** (zero data-dependent loops, conditional jumps, or early returns), the execution hot path triggers a `ReceiptRejected` refusal using strictly branchless arithmetic and bitwise masking:

1. **Branchless Polynomial Comparison**:
   Instead of control flow like `if actual == expected`, equality is evaluated using bitwise `XOR`. For inequalities like checking temporal bounds ($elapsed \ge required$), it uses two's-complement wrapping subtraction to extract the sign bit. This yields `1` (invalid) or `0` (valid) entirely without branches.
   ```rust
   // Example: const_lt_u32 returns 1 if premature, 0 if satisfied
   ((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
   ```

2. **Canonical Mask Generation**:
   The `0` or `1` condition output is scaled to a full integer mask via wrapping subtraction:
   ```rust
   // 0 - 1 = 0xFFFFFFFF (all 1s), 0 - 0 = 0x00000000 (all 0s)
   let active_mask = 0u32.wrapping_sub(condition & 1);
   ```

3. **Bitwise Aggregation and State Reversion**:
   The active mask selects the `ReceiptRejected` refusal bits and bitwise `OR`s (`union`s) them into a global `RefusalSet`. State mutations subsequently rely on a branchless multiplexer (e.g., `const_select_u32` or `select_nnf`). The presence of `ReceiptRejected` in the `RefusalSet` dynamically directs the multiplexer to reject the candidate mutation and clone the exact previous state bit-for-bit.
