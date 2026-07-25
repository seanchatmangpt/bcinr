### Exact Definition
`UnsupportedDomain` is a mandatory typed refusal (part of the `StabilityRefusal` enum) emitted when an authoritative operation or mathematical function receives an input that falls outside its strictly proven bounds—its "admitted domain". 

Common examples include calculating the logarithm of zero or executing a saturating division by zero. According to Rule 18 in `AGENTS.md`, the framework strictly forbids panicking, returning early, silently clamping outside the admitted policy, or returning a plausible default on unsupported inputs. Instead, the runtime must explicitly reject the unadmitted input via this typed refusal code.

### Branchless Mathematical Condition and Trigger Mechanism
To comply with BCINR's Radon Law (absolute branchlessness, CC=1), the condition is evaluated and propagated without any traditional control flow (like `if` statements). It follows a precise mathematical procedure:

1. **Branchless Condition Masking:** The boundary condition is evaluated using SWAR (SIMD Within A Register) and transformed into a full-width `CanonicalMask` (e.g., `0xFFFFFFFF` for true, `0x00000000` for false) instead of a standard boolean. Example: `let is_zero = const_eq_u32(self.val, 0);`
2. **Hardware Trap Avoidance:** The hot path completes the full calculation to avoid hardware panics. Using the mask, it branchlessly selects a safe fallback value to feed the mathematical pipeline if the domain is mathematically unsupported. Example: `let safe_val = is_zero.select_i32(-1048576, computed);`
3. **Sticky Fault Accumulation:** The condition mask branchlessly selects the `NumericFaultSet::INVALID_DOMAIN` fault bit (often combined with `DIVIDE_BY_ZERO`). This fault set acts as a join-semilattice; errors accumulate via bitwise union across chained calculations without "first-error-wins" short-circuiting.
4. **Sealed Return Types:** The function returns a fixed-width struct (like `SignedFixed` or `NonNegativeFixed`) that permanently pairs the safely computed dummy scalar with the accumulated error state (`NumericFaultSet`).
5. **State Selection & Translation:** Finally, at the absolute boundary of the hot path (e.g., during state admission in `allocator.rs`), the overall numeric fault set is evaluated. If the `INVALID_DOMAIN` bit is present, it is translated into the `StabilityRefusal::UnsupportedDomain` code. The operation is permanently rejected, and persistent state is left bit-for-bit unchanged via masked commits.
