# The `UnsupportedDomain` Typed Refusal in BCINR

Under **Rule 18** of the BCINR `AGENTS.md` constitution, all rejected authoritative operations must produce a bounded, strictly typed refusal code. Human-readable text and dynamic error messages are expressly forbidden in the hot path.

## Behavior on Inputs Outside the Bounded Domain

When a mathematical function or authoritative operation receives an input outside its rigorously defined and proven domain, the operation must reject the input by raising an `UnsupportedDomain` refusal. 

Due to the **Radon Law** ($CC=1$, absolute branchless execution), the runtime handles this scenario without traditional branching control flow (such as `if invalid { return Err(...); }`):

1. **Mask Generation:** The out-of-bounds condition is evaluated into a full-width bitmask via SWAR (SIMD Within A Register) and bitwise polynomials.
2. **State Selection:** Because Rule 10 ("No mutation before complete admission") prohibits speculative mutation, this mask is used in a constant-time `select` operation. The rejected operation leaves persistent state bit-for-bit unchanged.
3. **Fault Accumulation:** The mathematical invalidity is securely propagated (often by accumulating numeric faults in a bitwise registry) to the absolute boundary of the hot path.
4. **Refusal Translation:** The authoritative root securely translates the terminal state/mask into the legacy or strict `UnsupportedDomain` code, formally terminating the transaction without panicking or branching.

## Why Rule 18 Forbids Silent Clamping, Fallbacks, and Defaults

Rule 18 explicitly bans panicking, silent clamping (unless specifically bound by a Hoare contract), dropping factors, falling back to simpler algorithms, partial state mutations, or returning plausible defaults. These constraints enforce the "hard substrate" guarantees of BCINR:

1. **Axiomatic Determinism (Rule 1):** The core principle of BCINR is that admitted inputs map through a fixed instruction shape to deterministic outputs. Plausible defaults or silent truncation violate the mathematical contract defined by the independent oracle (`@hoare_oracle`), silently corrupting the formal proof obligations.
2. **State Integrity and Isolation (Rule 10):** Mutating partial state or clamping to an unverified boundary allows invalid states to bleed into persistent memory. A rejected mutation must guarantee zero side-effects.
3. **Architectural Branchlessness (Rules 3, 9, 12):** Falling back to a simpler algorithm or dropping a factor inherently requires runtime structural adaptation—which translates to hidden control-flow branches, algorithm search, or data-dependent execution. In BCINR, adaptive logic belongs to the offline "slow rail"; the hot path must execute only fixed-width, unrolled arithmetic.
4. **Adversarial Failure Law (Rule 4):** Under `@armstrong_fault` (Master of Failure Law), any corrupted or adversarial input (a "mutant") must hit a concrete, typed refusal boundary rather than generating a subtly wrong but plausible output. Strict typed refusals provide a mathematically sound surface area that guarantees all illegal states are caught structurally rather than heuristically.
