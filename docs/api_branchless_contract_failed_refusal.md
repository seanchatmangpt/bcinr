Based on the codebase analysis, here is how the `BranchlessContractFailed` refusal is implemented:

### 1. Conceptual vs. Implementation Identity
The exact identifier `BranchlessContractFailed` is purely a **CI gate and documentation concept** defined in `AGENTS.md` and `docs/branchless_contract_failed_refusal.md`. It does not exist verbatim in the source code. Instead, in the actual runtime (`crates/bcinr-cmca/src/allocator.rs`), this constitutional requirement is mapped to the **`StabilityRefusal::ContractViolation`** enum variant and its underlying **`RefusalSet`** bit-flags.

### 2. Branchless Generation (The Structural/Mathematical Checks)
The refusal is structurally enforced in `bcinr-cmca/src/allocator.rs` during the `allocate` function without any conditional control-flow (`CC=1`). Here is exactly how it is generated:

**A. Evaluation of Mathematical Envelopes:**
The runtime validates multiple Hoare constraints natively as boolean conditions (0 or 1), completely avoiding early `return` or `if` statements:
- `gd_ok`: Verifies contraction margins using `GAIN_MATRIX` and `WEIGHT_VECTOR`.
- `q_err`, `price_err`: Checks numeric bounds of lenses and prices.
- `lr_err`, `eta_err`, `beta_err`: Enforces learning rate, exploration floors, and maximum bounds.
- `dwell_err` & `digest_err`: Confirms timing and certificate digest integrity.

**B. Bit-Parallel Accumulation (`has_error`):**
These constraints are logically evaluated and accumulated purely via bitwise OR arithmetic:
```rust
let has_error = !gd_ok | digest_err | lr_err | beta_err | eta_err | dwell_err | q_err | price_err;
```

**C. Mask-Based Refusal Set Construction:**
Specific failure combinations are mapped to a `RefusalSet` using branchless `.masked()` selections and `.union()` accumulation. For instance, mathematical proposal rejections are merged into the set:
```rust
.union(RefusalSet::PROPOSAL_REJECTED.masked(((!gd_ok) | lr_err | beta_err | eta_err | q_err | price_err) as u32))
```
Additionally, structural violations like an empty candidate forest (`nl_is_zero`) are unconditionally unioned into the final `RefusalSet` via `RefusalSet::NO_LEAVES`.

**D. Mask-Based State Isolation:**
Rather than branching away on failure, candidate state mutations are discarded branchlessly using mask selection based on the refusal condition:
```rust
*last_switch_t = const_select_u32(has_refusal as u32, *last_switch_t, local_last_switch_t);
```

**E. Envelope Boundary Translation:**
At the boundary, `RefusalSet::primary_reason()` acts as the mechanical adapter. Any accumulated constraint failure that doesn't map to a narrow refusal (like `NO_LEAVES`), along with specific upstream failures (like `ROUND_MISMATCH`), falls through or explicitly matches to mechanically yield `StabilityRefusal::ContractViolation`:
```rust
} else if self.contains(Self::ROUND_MISMATCH) {
    StabilityRefusal::ContractViolation
// ...
} else {
    StabilityRefusal::ContractViolation
}
```
