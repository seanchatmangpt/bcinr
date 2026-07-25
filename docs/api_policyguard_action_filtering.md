I have located `PolicyGuard` and documented its branchless implementation for filtering `AutonomicAction`s in the Accept phase.

### Location
- **Implementation:** `/Users/sac/bcinr/crates/bcinr-logic/src/autonomic/policy_guard.rs`
- **Documentation:** `/Users/sac/bcinr/docs/gemini_mapek_accept_phase.md`

### Mathematical Mask Generation
In the BCINR architecture, the **Accept** step validates proposed `AutonomicAction`s strictly without conditional branching, satisfying the Radon Law ($CC=1$). `PolicyGuard` achieves this by translating safety boundaries (like risk matrices or thresholds) mathematically into fixed-width 64-bit masks.

For example, validating if a value exceeds a threshold is implemented entirely using arithmetic:
```rust
pub fn policy_guard_mask_gt(val: u64, threshold: u64) -> u64 {
    // 1. Cast the boolean result to an integer (1 if true, 0 if false)
    let check = (val > threshold) as u64;
    
    // 2. Expand to a full 64-bit mask via wrapping subtraction
    0u64.wrapping_sub(check)
}
```
* **If condition is True:** `check = 1`. `0 - 1` wraps around to `0xFFFFFFFFFFFFFFFF` (`!0`, or all ones).
* **If condition is False:** `check = 0`. `0 - 0` evaluates to `0x0000000000000000` (all zeros).

### Branchless Action Filtering
When an `AutonomicAction` mask is proposed, it is passed through the `PolicyGuard` for deterministic validation. 

Instead of conditional logic (e.g., `if is_safe() { accept() } else { reject() }`), the proposed action mask is filtered dynamically using bitwise arithmetic:
1. The `PolicyGuard` mask is evaluated against the proposed constraints.
2. A bitwise `AND` (`&`) is applied between the proposed action state and the `PolicyGuard` mask.
3. **Rejection:** If the policy boundary is violated, the guard mask evaluates to `0x0000000000000000`, mathematically zeroing out the proposed action.
4. **State Transition:** The Execution phase then updates the system using branchless multiplexing:
   ```rust
   next_state = (mask & proposed_state) | (~mask & current_state)
   ```

If the action is accepted (all ones), `proposed_state` is fully absorbed. If rejected (all zeros), the bitwise logic zeroes the proposed state and perfectly preserves `current_state` bit-for-bit without ever triggering a control-flow jump or early return.
