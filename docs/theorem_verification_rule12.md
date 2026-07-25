# Rule 12: Theorem Verification vs Discovery in BCINR

According to **Rule 12** of the BCINR Deterministic Substrate Constitution, the authoritative runtime (the hot path) is strictly prohibited from discovering or deriving stability parameters. Its sole responsibility is to execute deterministic, branchless verification of a fixed witness supplied by the "slow rail".

## 1. The Strict Separation of Concerns

*   **The Slow Rail (Theorem Discovery):** Responsible for executing complex, variable-iteration algorithms. It performs parameter derivation such as calculating the comparison matrix ($G$), the claimed positive witness vector ($d$), the contraction margin ($\delta$), and eigenvalue lower bounds. 
*   **The Hot Path / Authoritative Runtime (Theorem Verification):** Only verifies the mathematical correctness of these supplied parameters through branchless, fixed-width arithmetic. 

**Prohibited Runtime Operations:**
The following active discovery algorithms are strictly explicitly forbidden on the hot path:
*   Spectral-radius estimation
*   Power iteration
*   Jacobian derivation
*   Optimization over weighting vectors
*   Lyapunov search
*   Adaptive threshold discovery
*   Automatic q-range expansion
*   Dynamic graph analysis

## 2. Branchless Verification of Static Domination

The static domination law $G \cdot d \le (1 - \delta) \cdot d$ is evaluated branchlessly in `crates/bcinr-cmca/src/stability.rs` by the `derive_stability_candidate` function. 

### A. Fixed-Bounded Operations
To prevent unbounded iteration (Rule 13), the verification strictly operates over a compile-time fixed matrix dimension (`DIM = 2`).

### B. Fixed-Point Arithmetic and Overflow Prevention
No floating-point math is allowed. Inputs are encoded as Q16.16 scaled fixed-point integers. To safely compute the matrix-vector multiplication $G \cdot d$ branchlessly and avoid overflow checks, the calculations temporarily upscale to `i128` during the inner loop:

```rust
// G d, computed exactly in i128 to avoid overflow, then rescaled back to Q16.16.
let mut gd = [0i64; DIM];
for r in 0..DIM {
    let mut acc: i128 = 0;
    for c in 0..DIM {
        acc += (g[r][c] as i128) * (d[c] as i128);
    }
    gd[r] = (acc / SCALE as i128) as i64; // Rescaled to Q16.16
}
```

### C. Elementwise Domination Check
The constraint $(1 - \delta) d$ is calculated and bounded against $G \cdot d$. If the check fails, the execution explicitly yields a typed refusal:
```rust
return Err(StabilityDerivationRefusal::ContractionMarginInsufficient);
```

## 3. Cryptographic Sealing of the Theorem

Verification does not just yield a boolean assertion; it yields a cryptographic proof.
When the inequality holds mathematically, the runtime synthesizes a `candidate_digest` using a bitwise `mix64` hash. This process binds all the domain-specific parameters ($G$, $d$, $\delta$, noise radius, switch radius, etc.) into a sealed `StabilityCandidate` struct. This proves that the verification passed natively on the hot path and safely incorporates the matrix into the state machine.

```rust
// Excerpt of the cryptographic binding seal
let candidate_digest = StabilityCandidate::seal(
    g,
    d,
    margin_delta,
    noise_radius, // ... other parameters
);
```

## 4. Independent Recomputation

As an extra defense-in-depth measure defined in `certification.rs`, when minting a `CertificateReceipt` via `seal_certificate`, the code refuses to merely accept the candidate's previous boolean verification. It invokes `witness_holds(&candidate)` to **independently recompute** the static domination law directly from the candidate's inner fields. Failure yields a `CertificationRefusal::WitnessMarginInsufficient`. 

## 5. Branchless Verification of Eigenvalue Lower Bounds

Eigenvalue lower bounds are prohibited from runtime discovery. Instead:
1.  The slow rail provides the eigenvalue via a `MeasurementArtifact` (`artifact.gram_lower_bound`).
2.  The telemetry engine assesses the measurement branchlessly using bitwise operations (`const_lt_u32`) rather than branches:
    ```rust
    let gamma_under_off = const_lt_u32(gamma_min_plus_under.value_bits(), epsilon_gram.value_bits());
    ```
3.  The outcome is aggregated into flags (like `is_gram_degenerate`) purely via bitwise `&` logic, preserving a strict $CC = 1$ enforcement.
