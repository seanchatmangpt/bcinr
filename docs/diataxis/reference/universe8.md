# Universe8 Reference

**Universe8** is the register-scale deterministic substrate of the BCINR architecture. It acts as a mirror of Universe64 at an 8-base level, meaning that each computational tier aligns perfectly with register- or cache-line-scale memory architecture.

## Hierarchy & Cross-Tier Equivalence

| Power | Bits | Bytes | Role |
|---|---|---|---|
| $8^1$ | 8 | 1 | **Place atom** (one byte) |
| $8^2$ | 64 | 8 | **Cell** (one `u64` register) |
| $8^3$ | 512 | 64 | **Block** (one L1 cache line: `[u64; 8]`) |
| $8^4$ | 4096 | 512 | **Domain** (half memory page: `[u64; 64]`) |

Sixty-four instances of `U8Domain` (64 × 512 B = 32 KiB) are equivalent to exactly one Universe64 `UniverseBlock`.

Because they use the identical `FNV1A_64` offset basis and primes for execution receipts, U8 structures seamlessly interoperate with U64 receipts, allowing mixed-tier architectural verification.

## Timing Contract
Universe8 maintains even tighter bounds at the lower levels due to register-scale operations:

| Tier | Budget | Core Operations |
|---|---|---|
| **T0** | ≤ 8 ns | `U8Cell::fire`, `receipt_mix_u8_transition` |
| **T1** | ≤ 200 ns | `U8Block::fire_cell`, `U8Block::conformance_distance` |
| **T2** | ≤ 5 µs | `U8Domain::conformance_distance`, `delta` execution |

## Core Types
- **`U8Coord`**: Identifies a specific domain, cell, or place within the Universe8 hierarchy.
- **`U8Cell`**: A 64-bit component capable of firing an internal state machine (T0 operation).
- **`U8Block`**: Groups 8 cells. Aligns perfectly to one L1 cache line (64 bytes).
- **`U8Domain`**: Groups 8 blocks. Sits natively within memory pages.
- **`U8Receipt`**: Verifies transitions, identical in security properties to the U64 equivalents.

By using Universe8, the developer shifts the operational unit down to the cache line or CPU register, achieving sub-10ns ($T_0$) speeds with uncompromised mathematical certainty.