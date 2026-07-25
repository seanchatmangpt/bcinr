# Research Report: Rule 11 (ReceiptSound Law) and the Masked "Frozen Fallback"

Under **Rule 11 (The ReceiptSound Law)**, the BCINR deterministic substrate mandates that any adaptive state mutation is strictly gated by an irrefutable conjunction of mathematical proofs. One of the primary prerequisites is the **`CertifiedLearningMode`** receipt, which explicitly authorizes the system to transition from merely utilizing existing policies (inference) to actively adapting its state weights. 

If this receipt is missing, or if stability bounds are violated, the system is mathematically blocked from committing mutations. Instead, it must gracefully degrade into a **"frozen fallback"** state, where deterministic selection continues, but all adaptive state fields remain strictly unchanged. In compliance with **Rule 9 (Mask-based execution law)**, this fallback is implemented entirely via constant-time, branchless masked state selection.

Here is the exact mechanism observed in the `allocate` function (found in `crates/bcinr-cmca/src/allocator.rs`):

## 1. Authoritative Input and Mask Derivation

The authorization is injected as a fixed, deterministic input to the hot path using an `Option` containing the verified token:
```rust
proof: Option<&AdaptiveUpdate<CertifiedLearning>>
```
Instead of using an early return or `if proof.is_none()`, the system immediately derives boolean mask primitives:
```rust
let proof_some = proof.is_some();
let degrade_to_certified_selection = proof.is_none();
```

The system computes several control-plane validations (e.g., `gd_ok`, `dwell_err`, `lr_err`). It then evaluates the overall `update_allowed` mask:
```rust
let update_allowed = !(switch_wanted & !can_switch) & !freeze_learning & proof_some;
```
If `proof_some` is false (meaning the system is lacking the `CertifiedLearningMode` proof), `update_allowed` evaluates to `false` (0).

## 2. Masked State Selection for Weights

The adaptive `weights` update operates in an $O(N)$ unrolled loop. It computes the proposed mutations (exponentiating and distributing payoffs), but rather than branching to apply them, it utilizes `select_nnf` (a constant-time selection primitive over fixed-point values). 

When learning is frozen, the masks ensure an identity operation:
```rust
let is_updating = has_children & update_allowed; // evaluates to false
// ... computation of proposed flat and desc weights ...

local_weights[v & 7][(2 * q_idx) & 7] =
    select_nnf(is_updating as u32, w_flat * flat_signed.exp(), w_flat);
```
Because `is_updating as u32` is `0`, the selection ignores the exponentiated candidate and re-selects the original `w_flat` value, leaving the weight bit-for-bit unchanged. The subsequent normalization step is similarly masked by `update_allowed`, ensuring no accidental numeric drift.

## 3. Masked State Selection for Mode Switching

The selection of the dominant mode and timestamps is similarly governed by the fallback mask. The system checks if it *wants* to switch modes, but gates the actual application behind `did_switch`:
```rust
let did_switch = (new_dom_mode != local_prev_mode) & can_switch & !freeze_learning & proof_some;

local_last_switch_t = const_select_u32(did_switch as u32, t, local_last_switch_t);
local_prev_mode = const_select_u32(did_switch as u32, new_dom_mode, local_prev_mode);
```
Since `proof_some` is false, `did_switch` evaluates to `0`, causing `const_select_u32` to overwrite the local state variables with their existing unchanged values.

## 4. The Final Write-Back Gate

Finally, the `has_refusal` gate manages the commit phase. In a traditional branching model, an error would abort the transaction. Here, the write-back is unconditional, but the values written are masked:
```rust
let has_refusal = (has_error | (nl_is_zero != 0)) & !degrade_to_certified_selection;
```
Notice that if the system has gracefully degraded to selection-only (`degrade_to_certified_selection == true`), `has_refusal` evaluates to `false` (0), meaning it is *not* considered a hard error that halts execution. 

The unconditional commit then executes:
```rust
weights[v & 7][e & 7] = select_nnf(
    has_refusal as u32,
    weights[v & 7][e & 7],
    local_weights[v & 7][e & 7],
);
```
Because `has_refusal` is `0`, it selects `local_weights`. And because `is_updating` was `0` earlier, `local_weights` is mathematically identical to the original `weights`. The entire pipeline executes fully, consuming identical CPU cycles regardless of whether learning is active or frozen, fulfilling the branchless bounds required by Rule 11.
