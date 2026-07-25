### Findings
1. **Absence in Source Code**: The `ReceiptRejected` typed refusal is actually **NOT** found anywhere in `crates/bcinr-cmca/src/` or `crates/bcinr-api/src/`. It does not appear in any `RefusalSet` bitmask or `StabilityRefusal` enum in the implementation, nor does it appear in any other crate.
2. **Presence in Documentation**: However, it is heavily documented in the `docs/` directory (`docs/receipt_rejected_refusal.md`, `docs/receipt_rejected_temporal.md`, and `docs/rule_18_receipt_rejected_refusal.md`) as a required category under Rule 18 of `AGENTS.md`.

### The Mathematical / Structural Check
Based on the documentation (and mirrored by existing bitmask refusals like `DWELL_UNSATISFIED` in `bcinr-cmca`), here is the branchless mathematical check that produces these types of refusals:

1. **Branchless Polynomial Comparison**:
   To evaluate whether a receipt satisfies its constraints (such as a temporal window bound `elapsed >= required`), it relies on bitwise sign extraction (two's-complement arithmetic) rather than control flow like `if`:
   ```rust
   // const_lt_u32 returns 1 if premature, 0 if satisfied
   ((a_bb ^ ((a_bb ^ b_bb) | (a_bb.wrapping_sub(b_bb) ^ b_bb))) >> 31) & 1
   ```
   For structural equality (like digest matching), it evaluates using bitwise XOR, combining multiple checks using `OR`. A result of `0` means all fields matched.

2. **Canonical Mask Generation**:
   The output `0` or `1` is then scaled to a full integer mask using wrapping subtraction:
   ```rust
   // condition & 1 evaluates to 1 (invalid) or 0 (valid)
   // 0 - 1 = 0xFFFFFFFF (all 1s), 0 - 0 = 0x00000000 (all 0s)
   self.0 & 0u32.wrapping_sub(condition & 1)
   ```

3. **Bitwise Aggregation and State Reversion**:
   This active mask selects the typed refusal bits (like the conceptual `RECEIPT_REJECTED`) and unions them into a global `RefusalSet`. State mutations subsequently use a multiplexer (`select_nnf` or `const_select_u32`) to commit the candidate state only if the refusal mask is `0`, falling back seamlessly to the previous state without branching.
