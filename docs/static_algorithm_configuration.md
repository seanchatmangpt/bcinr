# Static Algorithm Configuration in BCINR

According to the deterministic substrate constitution (Rule 3), `bcinr` strictly prohibits **runtime algorithm search**. This ensures a fixed instruction shape ($CC=1$) and guarantees bounded execution work. The codebase implements static configuration instead of runtime search using a combination of **Slow Rail derivation**, **Constant-time Witness Verification**, and **Mask-based Execution**.

## 1. Slow Rail Derivation vs. Hot Path Verification

Instead of dynamically discovering the optimal algorithm, structural bounds, or stability parameters on the fly, the system separates these concerns:
- **The Slow Rail:** Non-authoritative, branching execution is used to search for structural bounds, weighting vectors, and stability certificates.
- **The Hot Path:** The authoritative runtime is passed these pre-derived parameters as a **static configuration** (a fixed witness). It strictly *verifies* this configuration using constant-time packed value comparisons and never attempts to dynamically derive it.

## 2. Static Algorithm Configuration Structs

The static algorithms and bounds are encapsulated in structures that are evaluated branchlessly:

### `StabilityCandidate`
This struct acts as the mathematical witness for learning algorithm stability. It contains:
- The certified transition matrix bounds ($G$) and vector ($d$).
- The expected margin of contraction ($\delta$).
- The runtime verifies static domination ($G \cdot d \leq (1 - \delta)d$) in constant time using `witness_holds()` by computing over fixed arrays (e.g., `DIM` size) and ensuring bounded numeric error. 

### `CertificateBindings`
Rather than having an `if/else` tree to decide which structural rules to apply, the system defines eleven domain-specific static bindings that comprise the exact algorithm and structural configuration for a transition. Any deviation results in a typed refusal.
```rust
pub struct CertificateBindings {
    pub admitted_graph: u64,
    pub generated_payload: u64,
    pub kernel_specialization_identity: u64,
    pub numeric_profile: u64,
    pub q_registry: u64,
    pub pricing_law: u64,
    pub floor_law: u64,
    pub control_mode: u64,
    pub influence_state: u64,
    pub comparison_derivation: u64,
    pub round_identity: u64,
}
```
All fields are verified as identities/digests. The codebase verifies that the candidate's actual environment exactly matches these pre-computed identities.

## 3. Mask-Based "Frozen" Fallback (No Silent Refusals)

If the supplied static bounds are violated, the system does not search for a degraded fallback algorithm. Instead, it relies on:
- **Typed Refusals:** Returning constant structures like `CertificationRefusal::WitnessMarginInsufficient` or `ContractionMarginInsufficient`.
- **Masked State Selection:** In `CertifiedLearningMode`, if bounds are breached, the admission mask evaluates to `0`. The runtime branchlessly commits the fallback state via $x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$, where a mask of `0` strictly enforces bit-for-bit immutability, elegantly freezing the learning mode without `if` or `match` conditions.
