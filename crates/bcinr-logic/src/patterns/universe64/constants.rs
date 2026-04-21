//! # Universe64 Geometry & Dual-Plane Execution Constants
//!
//! Named constants defining the fixed shape of a `UniverseBlock` and its
//! Dual-Plane execution envelope (Data Plane + Scratch Plane).
//!
//! Geometry: 64 domains × 64 cells × 64 places = 4096 words × 8 bytes = 32 KiB.
//! Envelope: 32 KiB resident Data Plane + 32 KiB Scratch Plane = 64 KiB L1 target.

/// Number of top-level domains in a `UniverseBlock`.
pub const DOMAIN_COUNT: usize = 64;

/// Number of cells per domain in a `UniverseBlock`.
pub const CELL_COUNT: usize = 64;

/// Number of boolean place bits within a single cell word.
pub const PLACE_COUNT: usize = 64;

/// Total number of `u64` words in a `UniverseBlock` state array.
pub const UNIVERSE_WORDS: usize = DOMAIN_COUNT * CELL_COUNT;

/// Size in bytes of a fully-populated `UniverseBlock` (the Data Plane).
pub const SIZE_BYTES: usize = UNIVERSE_WORDS * 8;

/// Number of bits required to represent one coordinate axis index (0..63).
pub const COORD_BITS: u32 = 6;

/// Bitmask for clamping a value to one coordinate axis range `[0, 63]`.
pub const COORD_MASK: u32 = 0x3F;

/// Bit shift for domain field in packed UCoord (domain occupies bits [17:12]).
pub const DOMAIN_SHIFT: u32 = COORD_BITS * 2;

/// Bit shift for cell field in packed UCoord (cell occupies bits [11:6]).
pub const CELL_SHIFT: u32 = COORD_BITS;

/// Bit shift for place field in packed UCoord (place occupies bits [5:0]).
pub const PLACE_SHIFT: u32 = 0;

/// Mask for extracting word index (domain<<6 | cell) from packed UCoord.
pub const WORD_INDEX_MASK: u32 = 0xFFF;

/// Total number of boolean places in one Universe: 64^3.
pub const TOTAL_PLACES: usize = DOMAIN_COUNT * CELL_COUNT * PLACE_COUNT;

/// Maximum valid flat word index (inclusive).
pub const MAX_WORD_INDEX: usize = UNIVERSE_WORDS - 1;

/// FNV-1a 64-bit offset basis. Use as initial `receipt` value for a fresh hash.
pub const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf29ce484222325;

/// FNV-1a 64-bit prime multiplier.
pub const FNV1A_64_PRIME: u64 = 0x100000001b3;

// ---------------------------------------------------------------------------
// Dual-Plane L1 Envelope (C4 architecture)
// ---------------------------------------------------------------------------

/// Size in bytes of the Scratch Plane (bounded motion workspace).
pub const SCRATCH_BYTES: usize = 32_768;

/// Total L1D-class envelope: Data Plane + Scratch Plane.
pub const L1_ENVELOPE_BYTES: usize = SIZE_BYTES + SCRATCH_BYTES;

/// Maximum active word count for T1 sparse microkernel scope.
pub const ACTIVE_WORD_CAPACITY: usize = 64;

// ---------------------------------------------------------------------------
// Tier budgets (wall-clock ceilings, nanoseconds)
// ---------------------------------------------------------------------------

/// T0 primitive tier budget: single-word/bit truth atom.
pub const T0_BUDGET_NS: u64 = 2;

/// T1 scoped microkernel tier budget: bounded active words.
pub const T1_BUDGET_NS: u64 = 200;

/// T2 full-universe tier budget: 4096-word scans.
pub const T2_BUDGET_NS: u64 = 5_000;

/// T3 control-epoch tier budget (UniverseOS scheduling boundary).
pub const T3_BUDGET_NS: u64 = 10_000;

/// T4 sentinel: external boundary (human / API / dashboard). No internal budget.
pub const T4_EXTERNAL: u64 = u64::MAX;

// ---------------------------------------------------------------------------
// PHD integrity gate probes (per-type)
// ---------------------------------------------------------------------------

pub const UNIVERSEBLOCK_GATE: u64 = 0x6464646464646464;
pub const UNIVERSEDELTA_GATE: u64 = 0xDE17ADE17ADE17A0;
pub const UCOORD_GATE: u64 = 0x000000000C000D00;
pub const URECEIPT_GATE: u64 = 0xECE1E1E1ECE1E1E1;
pub const UMASK_GATE: u64 = 0x0000000000000001;
pub const UTRANSITION_GATE: u64 = 0x78A717105017001;
pub const USCRATCH_GATE: u64 = 0x5C7A7C4C57415CA0;
pub const UINSTRUCTION_GATE: u64 = 0x1A57717C710A5701;

// ---------------------------------------------------------------------------
// Compile-time consistency assertions
// ---------------------------------------------------------------------------

const _: () = assert!(UNIVERSE_WORDS == DOMAIN_COUNT * CELL_COUNT);
const _: () = assert!(SIZE_BYTES == UNIVERSE_WORDS * 8);
const _: () = assert!(COORD_MASK == 0x3F);
const _: () = assert!(DOMAIN_SHIFT == 12);
const _: () = assert!(CELL_SHIFT == 6);
const _: () = assert!(PLACE_SHIFT == 0);
const _: () = assert!(WORD_INDEX_MASK == 0xFFF);
const _: () = assert!(TOTAL_PLACES == 262_144);
const _: () = assert!(MAX_WORD_INDEX == 4095);
const _: () = assert!(FNV1A_64_OFFSET_BASIS == 14695981039346656037u64);
const _: () = assert!(FNV1A_64_PRIME == 1099511628211u64);
const _: () = assert!(SCRATCH_BYTES == 32_768);
const _: () = assert!(L1_ENVELOPE_BYTES == 65_536);
const _: () = assert!(T0_BUDGET_NS < T1_BUDGET_NS);
const _: () = assert!(T1_BUDGET_NS < T2_BUDGET_NS);
