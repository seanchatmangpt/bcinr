# MFW Auto-Select Deterministic Chaos Synchronization Oracle

**Owner:** `@hoare_oracle`
**Jurisdiction:** `mfw-auto-select` authoritative hot path (`/Users/sac/mfw/mfw-auto-select/src/chaos.rs`)

This document defines the strict mathematical laws, Hoare contracts, valid domains, and proof obligations for integrating zero-allocation deterministic chaos synchronization into the CMCA `mfw-auto-select` token routing mechanics, in accordance with the BCINR Deterministic Substrate Constitution (`AGENTS.md`).

---

## 1. Mathematical Law and Execution Domain

The objective of deterministic chaos synchronization in auto-select routing is to inject a bounded, deterministic pseudo-random perturbation (chaos) into the token routing mass to prevent degenerate limit cycles and stagnation in tool selection, without violating the `CC=1` branchless execution law and maintaining zero allocation.

Let $S_{base}$ be the base token mass for a candidate $c_i$.
Let $K_{sync} \in [0, 2^{64}-1]$ be the synchronized chaos key (derived from the current envelope receipt or epoch digest).
Let $i \in [0, 7]$ be the topological candidate index.

The chaos perturbation function $P(K_{sync}, i)$ must compute a deterministic fixed-point multiplier $M_{chaos} \in [1.0 - \delta, 1.0 + \delta]$ using integer arithmetic, where $\delta$ is the maximum chaos amplitude (represented in Q16.16 fixed-point).

The synchronized mass is computed as:
$$ S_{sync}(i) = S_{base}(i) \otimes P(K_{sync}, i) $$

Where $\otimes$ is a saturating fixed-point multiplication.

---

## 2. Hoare Contracts

For every primitive involved in chaos synchronization, a strict Hoare contract $\{P(x)\} \quad f(x) \quad \{Q(x,f(x))\}$ is enforced.

### 2.1 Chaos Multiplier Derivation (`derive_chaos_multiplier`)

**Mathematical Law:** 
Computes a bounded perturbation multiplier from the synchronized key and index using a branchless mix function.
$$ H_i = \text{mix64}(K_{sync} \oplus (i \times \text{PRIME})) $$
$$ M_{chaos} = \text{BASE\_MULTIPLIER} + (H_i \pmod{2\Delta}) - \Delta $$

**Hoare Contract:**
* **Valid Input Domain:** $K_{sync} \in [0, 2^{64}-1]$, $i \in [0, 7]$, $\Delta \in [0, 2^{15}]$.
* **Output Range:** $M_{chaos} \in [\text{BASE\_MULTIPLIER} - \Delta, \text{BASE\_MULTIPLIER} + \Delta]$.
* **Conservation Law:** Identical $K_{sync}$ and $i$ yields identically bit-exact $M_{chaos}$.
* **Monotonicity Law:** N/A (chaotic by design).
* **Overflow Behavior:** $H_i \pmod{2\Delta}$ strictly bounds the output to a displacement $\le \Delta$. No overflow possible in 64-bit arithmetic.
* **Invalid-Input Refusal:** None. Total over the $u64$ and $u8$ domain.
* **Determinism:** Branchless (CC=1) static bitwise XOR, multiplication, and modulus.
* **State-Mutation Boundary:** 0 heap allocations, pure stack evaluation.
* **Numeric Error Envelope:** Exact integer mathematics. $E_{abs} = 0$.

### 2.2 Synchronized Token Mass (`apply_chaos_sync`)

**Mathematical Law:**
Applies the fixed-point chaos multiplier to the base mass with saturating bounds.
$$ S_{sync} = \min(S_{max}, \max(0, (S_{base} \times M_{chaos}) \gg 16)) $$

**Hoare Contract:**
* **Valid Input Domain:** $S_{base} \in [0, 255]$, $M_{chaos}$ from derivation.
* **Output Range:** $S_{sync} \in [0, 255]$.
* **Conservation Law:** If $\Delta = 0$, $S_{sync} = S_{base}$.
* **Monotonicity Law:** For a fixed $M_{chaos}$, $S_{base_1} \ge S_{base_2} \implies S_{sync_1} \ge S_{sync_2}$.
* **Overflow Behavior:** 32-bit multiplication $255 \times (2^{16} + \Delta)$ fits easily in $u32$. Right shift safely scales back.
* **Invalid-Input Refusal:** Handled at higher levels; internal arithmetic is total.
* **Determinism:** Branchless (CC=1) multiplication and bitwise shifts. Min/max evaluated via bit-parallel SWAR or equivalent branchless logic.
* **State-Mutation Boundary:** Fixed inputs, pure value return. Zero allocation.
* **Numeric Error Envelope:** $E_{abs} < 1.0$ due to truncation from right shift.

---

## 3. Proof Obligations

To ensure rigorous integrity of the chaos synchronization implementation, the following independent proof obligations must be certified:

1. **Topological Loop Freedom (@turing_machine):**
   Must formally verify that the generated assembly for `derive_chaos_multiplier` and `apply_chaos_sync` contains exactly zero loop backedges, zero dynamic branch instructions (no `je`, `jne`, `jl`, etc.), and zero allocator calls.
2. **Exhaustive Synchronization Matrix (@hoare_oracle):**
   Using property testing, prove that for all combinations of boundary $S_{base}$ and representative $K_{sync}$, the branchless bitwise logic yields identical outputs to an explicitly branching arithmetic oracle bounding the values.
3. **Refusal Adherence (@armstrong_fault):**
   Assert typed refusals (e.g. `NumericRangeExceeded`) trigger if chaos parameters exceed statically permitted bounds at the API boundary.
4. **Mutant Survivability (@armstrong_fault):**
   Structural mutants (e.g., omitting the right shift, dropping the modulo, incorrect bitwise clamp) must unconditionally fail the independent test oracle. SIS score drops to 0 on any survival.
