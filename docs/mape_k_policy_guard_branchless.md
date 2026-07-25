# Branchless Gating in the MAPE-K Loop (Accept Phase)

The `bcinr` substrate enforces a strict branchless execution model (the Radon Law, CC=1), meaning no data-dependent control flow (like `if` or `match` statements) is permitted in the authoritative hot path. 

In the autonomic **MAPE-K** (Monitor, Analyze, Plan, Execute, Knowledge) loop, the **Accept** phase must decide whether to permit a proposed `AutonomicAction`. Instead of branching logic, this decision is implemented mathematically using `PolicyGuard` (`crates/bcinr-logic/src/autonomic/policy_guard.rs`).

## 1. Mathematical Mask Generation
The `PolicyGuard` translates boolean boundaries into full-width 64-bit bitmasks. It does this strictly through arithmetic, generating `!0` (`0xFFFFFFFFFFFFFFFF`) for a true condition and `0x0` (`0x0000000000000000`) for false:

```rust
pub fn policy_guard_mask_gt(val: u64, threshold: u64) -> u64 {
    let check = (val > threshold) as u64; // Evaluates to 1 or 0
    0u64.wrapping_sub(check)              // 0 - 1 wraps to all 1s (!0), 0 - 0 = 0
}
```

The underlying struct offers `mask_gt`, `mask_lt`, and `mask_eq`, all relying on `wrapping_sub` to yield a boolean-derived mask without ever generating a jump instruction.

## 2. Mask-Based Execution (The Accept Phase)
During the Accept phase, `PolicyGuard` uses the generated mask to evaluate the safety/threshold of the proposed action against system conditions. 

Following the `bcinr` mask-based execution law, the Accept phase commits the state transition through bitwise selection rather than speculative mutation or early returns:

$$ x_{t+1} = (m_{\mathrm{admitted}} \land x_{\mathrm{candidate}}) \lor (\neg m_{\mathrm{admitted}} \land x_t) $$

In Rust, this branchless selection looks like:
```rust
// Evaluate action against the policy mathematically
let mask = PolicyGuard::mask_lt(risk, threshold);

// Conditionally apply the state using bitwise logic, executing both branches simultaneously
let next_state = (mask & candidate_state) | (!mask & current_state);
```

### Architectural Benefits:
1. **Timing-Attack Immunity**: Because both the accepted and rejected paths collapse into identical arithmetic instructions, they take exactly the same number of CPU cycles.
2. **Zero-Allocation Boundary**: Generates no dynamic jumps and requires 0 allocations, conforming to the absolute `bcinr` runtime laws.
3. **Formal Verifiability**: Transforms sequential semantic decisions directly into bitwise polynomials (SWAR), enabling exhaustive bit-vector proof verification for autonomic system invariants.
