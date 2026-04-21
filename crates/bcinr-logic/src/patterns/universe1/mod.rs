//! # Universe1_n — Bit-Native Operating Substrate
//!
//! `U_{1,n} = 𝔹ⁿ`. A universe of `n` one-bit truth atoms.
//!
//! `U1_n` is **not** a bit-width integer type. It is a substrate where the
//! atomic unit is one Boolean truth coordinate. `n` is the atom count; `n/8`
//! is the byte footprint; both derive from the formal name. Bytes are a
//! consequence, not a primitive.
//!
//! Canonical profiles defined in this release:
//!
//! | Type       | Formal         | Atoms (n)   | Bytes   | Role                             |
//! |------------|----------------|-------------|---------|----------------------------------|
//! | `U1_8`     | `U_{1,8}`      | 8           | 1       | Place atom (alias for `u8`)      |
//! | `U1_64`    | `U_{1,64}`     | 64          | 8       | Cell — one `u64` register        |
//! | `U1_512`   | `U_{1,512}`    | 512         | 64      | Block — one L1 cache line        |
//! | `U1_4096`  | `U_{1,4096}`   | 4096        | 512     | Domain — half a 4 KiB page       |
//!
//! The 64-tier profiles `U_{1,64³}` (262,144 atoms / 32 KiB, attention × truth)
//! and `U_{1,64⁴}` (16,777,216 atoms / 2 MiB, meaning field) are provided
//! under the existing `universe64` module pending full migration.
//!
//! ## Cross-Profile Composition
//!
//! 64 × `U1_4096` = `U_{1,64³}` (32 KiB). The atom count composes; the kernel
//! law does not change. The same `FNV1A_64` constants underpin both receipt
//! systems, so receipts may be mixed across profiles without re-seeding.
//!
//! ## Timing Contract
//!
//! | Tier | Budget   | U1 Operation                                           |
//! |------|----------|--------------------------------------------------------|
//! | T0   | ≤ 8 ns   | `U1_64::fire`, `receipt_mix_u1_transition`             |
//! | T1   | ≤ 200 ns | `U1_512::fire_cell`, `U1_512::conformance_distance`    |
//! | T2   | ≤ 5 µs   | `U1_4096::conformance_distance`, `U1_4096::delta`      |
//!
//! > Bits, not bytes. Truth, not data. One atom, `n` lawful worlds.

pub mod block;
pub mod cell;
pub mod constants;
pub mod coord;
pub mod domain;
pub mod receipt;
pub mod transition;

pub use block::U1_512;
pub use cell::U1_64;
pub use constants::*;
pub use coord::U1Coord;
pub use domain::U1_4096;
pub use receipt::{new_u1_receipt, receipt_mix_u1_transition, U1Receipt};

/// `U1_8` — the 8¹ atom: one byte (8 bits / one Place).
///
/// ```
/// use bcinr_logic::patterns::universe1::U1_8;
/// let x: U1_8 = 0xAB;
/// assert_eq!(x, 0xAB);
/// ```
pub type U1_8 = u8;
