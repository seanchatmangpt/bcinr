# Anti-Patterns in Branchless Calculus

## 1. Branching in Latency-Critical Loops
**Anti-Pattern:** Using `if` or `match` inside loops for performance-critical pathing.
**Consequence:** Branch predictor state pollution, pipeline stalls, and timing side-channels.
**Correction:** Utilize `Mask` or `Select` family primitives to map logical branches into arithmetic data-paths.

## 2. Unchecked Arithmetic in Physical Actuation
**Anti-Pattern:** Direct `+` or `-` operators for inputs controlling physical actuators.
**Consequence:** Arithmetic wrap-around (overflow) causes fatal physical failures (e.g., inverting thrust or cooling levels).
**Correction:** Use the `Fix` family primitives (`add_sat`, `sub_sat`) which are saturation-aware and branchless by design.

## 3. Cache-Line False Sharing
**Anti-Pattern:** Declaring shared buffers or transition tables without explicit 64-byte alignment (`#[repr(align(64))]`).
**Consequence:** High cache-invalidation traffic during concurrent access, leading to multi-lane latency spikes.
**Correction:** Use cache-line alignment on all shared/static state transition structures.

## 4. O(N) Spatial Queries
**Anti-Pattern:** Linear scanning of dense state arrays for bitmask operations.
**Consequence:** Throughput degradation at high scale ($>10^6$ agents).
**Correction:** Utilize `rank_u64` and `select_bit_u64` for constant-time $O(1)$ spatial coordinate resolution.

## 5. Self-Canceling XOR Expressions
**Anti-Pattern:** Writing `A.wrapping_add(B) ^ A` where the XOR operand duplicates earlier computation.
**Consequence:** Logic is completely erased (XOR is self-inverse); function body becomes meaningless. Tests pass because reference implementation is identical to the cheating implementation.
**Correction:** Implement the actual algorithm. Use `bcinr-cheat-scanner` to detect and block this pattern in CI (`cargo make scan-cheats`).

## 6. Circular Reference Oracles
**Anti-Pattern:** Test reference function is a verbatim copy of the implementation (`_reference` = `impl`).
**Consequence:** Equivalence tests (`assert_eq!(reference(x) == impl(x))`) always pass regardless of correctness. Cannot falsify wrong code.
**Correction:** Write a semantically independent reference using standard (possibly branching) algorithms. Ensure reference and implementation differ in structure.

## 7. Magic Constants as Scaffolding
**Anti-Pattern:** Hardcoded constants like `0xDEADBEEF`, `0xCAFEBABE` in production code outside test modules.
**Consequence:** Obscures domain semantics; indicates incomplete or test-driven code left in production. Hash functions should use FNV primes, not test markers.
**Correction:** Replace with domain-appropriate constants. Hash functions use primes (e.g., FNV-1a offset basis); encoders use standards-defined values (RFC). Use `bcinr-cheat-scanner` to enforce.

## 8. Artificial File-Length Inflation
**Anti-Pattern:** Adding numbered comment blocks (`// 1. Line 1`, `// 2. Line 2`, etc.) to meet an arbitrary minimum file size (e.g., ≥100 lines).
**Consequence:** Bloats codebase; hides actual code-to-comment ratio; signals process-driven rather than semantically-driven documentation.
**Correction:** Document only when value exists. If genuine documentation would exceed minimum, the implementation is probably incomplete anyway. Use `bcinr-cheat-scanner` to detect.

## 9. Boilerplate Verification Claims
**Anti-Pattern:** Copy-pasting identical "Hoare-logic Verification Line N: Branchless path is the unique solution..." comments across 20+ files without substantive proof.
**Consequence:** Claims formality without delivering it; audit trail becomes noise. Readers cannot distinguish real proofs from padding.
**Correction:** Write proofs specific to each primitive using actual Hoare-triple syntax (precondition, postcondition, invariant). See `docs/diataxis/reference/phd_gates.md` for format. Use `bcinr-cheat-scanner` to flag boilerplate.
