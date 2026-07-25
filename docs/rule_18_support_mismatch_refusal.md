Here is the detailed research on the `SupportMismatch` typed refusal based on `AGENTS.md` (Rule 18) and the `support_mismatch_refusal.md` documentation:

### What is `SupportMismatch`?

`SupportMismatch` is one of the mandatory typed refusals required by **Rule 18** of the BCINR constitution (`AGENTS.md`). In the BCINR deterministic substrate, unsupported inputs cannot trigger panics, early returns, or silent state mutations. Instead, deviations from mathematically proven boundaries must produce a bounded typed refusal like `SupportMismatch`.

Because BCINR mandates strict branchless execution (the Radon Law, $CC=1$), `SupportMismatch` is not thrown using control flow like `if !valid { return Err(SupportMismatch); }`. Instead, it is represented structurally as a bitwise polynomial and SWAR (SIMD Within A Register) mask.

### When is it surfaced in the runtime?

It is surfaced when evaluating bounded data structures, resource constraints, or topological graphs (such as partially ordered workflow graphs). Specifically, it occurs when an incoming transition or allocation request attempts to access or request nodes that fall outside the statically admitted topology mask (the operational boundaries).

### How it is processed branchlessly:

1. **Mask Derivation:** 
   The bounds check is calculated in bit-parallel, generating a full-width Canonical Mask (e.g., `0xFFFFFFFFFFFFFFFF` for a mismatch).
2. **Safe Fallback Execution:**
   To guarantee constant-time execution without early returns, a branchless multiplexer (`select(mask, fallback, active)`) routes the computation to a safe, fixed fallback state (like a null op). The pipeline processes fully regardless of validity.
3. **Sticky Fault Accumulation:**
   The fault does not halt execution. The `mismatch_mask` selects the `SUPPORT_MISMATCH` bit and logically `OR`s it into a bitwise fault accumulator (e.g., `TopologyFaultSet`). This accumulation acts as a join-semilattice, operating in strict constant time.
4. **Masked Commit & Rejection:**
   At the outer boundary of the authoritative hot path, the accumulated transaction mask is evaluated. If the `SUPPORT_MISMATCH` fault bit is set, the state transition is rejected using a field-wise masked commit ($x_{t+1} = \operatorname{select}(m_{\mathrm{admitted}}, x_{\mathrm{candidate}}, x_t)$). The persistent state remains bit-for-bit unchanged.
5. **API Boundary Translation:**
   Finally, at the safe API boundary (outside the hot path), the accumulated fault bit is translated into the `Err(StabilityRefusal::SupportMismatch)` enumeration for the upstream caller.
