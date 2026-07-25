# Rule 22: Architecture-Specific Instructions (`PDEP`/`PEXT`) and the Target Matrix

In the deterministic `bcinr` substrate, architecture-specific instructions like `PDEP` (Parallel Bit Deposit) and `PEXT` (Parallel Bit Extract) are heavily used to evaluate logic in parallel without branches (satisfying the **Radon Law ($CC=1$)**). According to **Rule 22 (Feature and target matrix)**, incorporating these instructions requires strict compile-time gating, lawful branchless fallbacks, and comprehensive cross-target verification.

## 1. Feature Flag & Target Configuration Isolation

Architecture-specific intrinsics are isolated using a strict conditional compilation structure. 

Instead of relying on runtime CPU feature detection (which would introduce branches), `bcinr` uses static compile-time dispatch to route primitives to the correct hardware path:
- The fast-path logic is gated behind specific architecture and target feature flags, e.g., `#[cfg(target_arch = "x86_64")]` combined with `#[target_feature(enable = "bmi2")]` or `#[target_feature(enable = "sse4.2,ssse3")]`.
- A mutually exclusive scalar fallback is defined using `#[cfg(not(any(...)))]` for unsupported architectures.

This ensures that the exact target features are evaluated statically, completely removing runtime selection overhead and branching.

## 2. Guaranteeing a Lawful Fallback

When a target architecture lacks native hardware support for `PDEP` or `PEXT`, a naïve software fallback (like a standard bit-by-bit `while` loop with an `if` condition) is **strictly prohibited**. Standard loops introduce Jump Conditional Code (JCC) violations and timing side-channels, fundamentally breaking the $CC=1$ rule.

A lawful fallback is guaranteed through two strictly enforced mechanisms:

### A. The Branchless Algorithmic Fallback
The compliant software fallbacks (e.g., `expand_bits_u64.rs` for `PDEP` and `compress_bits_u64.rs` for `PEXT`) utilize **fully unrolled, 6-stage constant-time parallel-prefix algorithms** (adapted from *Hacker's Delight*). 
- These fallbacks consist purely of primitive bitwise arithmetic (`^`, `|`, `&`, `<<`, `>>`).
- They execute in strictly constant time regardless of data inputs, structurally matching the $CC=1$ deterministic guarantees of the hardware intrinsics.

### B. Typed Refusal (The Ultimate Guardrail)
If an $O(1)$ constant-time software fallback cannot be successfully constructed for a specific instruction on an unsupported target, the runtime must not panic, silently truncate, or degrade into a branching loop. Instead, the primitive must immediately yield a deterministic **Typed Refusal** (e.g., `Err(SupportMismatch)`). This guarantees that the system's execution shape never silently violates the deterministic runtime laws.

## 3. The Target Matrix Architecture

Rule 22 mandates that passing a single feature configuration is insufficient for repository standing. Every primitive must be verified across an exhaustive matrix of combinations to ensure the absence of branches across all compilation paths. 

The complete target matrix that all structural gates (such as the object-code audit and cheat scanner) must run across includes:

1. `default features`
2. `no default features`
3. `all features`
4. `release profile`
5. `supported architectures` (testing both native targets and their fallback paths)
6. `test profile where relevant`
7. `generated clean tree`

Furthermore, each individual compilation path (both the native hardware intrinsic path and its software fallback path) requires its own **separate disassembly evidence** during the `audit-object-code` step to definitively prove that neither compilation path introduced an implicit conditional jump, panic path, or loop backedge into the final released machine code.
