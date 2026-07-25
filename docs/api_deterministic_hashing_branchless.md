# Deterministic Hashing in BCINR: Branchless Execution and Side-Channel Avoidance

The `bcinr-logic` crate provides deterministic hashing algorithms that adhere strictly to the project's branchless architecture (`CC=1`, zero-allocation, bounded execution). Found in `crates/bcinr-logic/src/algorithms/`, these implementations use purely mathematical and bitwise operations to eliminate timing side-channels and data-dependent control flow.

Here is an analysis of three prominent hashing primitives and how they maintain fixed execution width branchlessly:

## 1. `crc32c_branchless` (CRC-32C Castagnoli)
**Location:** `crates/bcinr-logic/src/algorithms/crc32c_branchless.rs`  
**Tier:** T1 — sequential byte stream primitive

### Fixed Execution Width & Branchless Design:
- **Compile-Time Table Generation:** The 256-entry lookup table is generated via a `const fn` at compile-time. Even the generator avoids branching by computing a fixed mask `let mask = (crc & 1).wrapping_neg();` (which evaluates to either `0xFFFFFFFF` or `0x00000000`) and applying it via XOR: `(crc >> 1) ^ (CRC32C_POLY & mask)`. 
- **Constant-Time Loop:** The main runtime loop is completely devoid of `if` statements. Each byte is processed using exactly the same sequence of instructions:
  ```rust
  let idx = ((crc ^ byte as u32) & 0xFF) as usize;
  crc = CRC32C_TABLE[idx] ^ (crc >> 8);
  ```
- **Side-Channel Avoidance:** Because every byte is hashed through exactly one table lookup, one shift, and two bitwise XORs, there are no data-dependent execution paths. An attacker cannot use timing differentials to deduce the input, as the processor executes identical machine instructions regardless of the byte values.

## 2. `wyhash_64` (WyHash v4)
**Location:** `crates/bcinr-logic/src/algorithms/wyhash_64.rs`  
**Tier:** T2 — streaming hash primitive

### Fixed Execution Width & Branchless Design:
- **The `wymix` Primitive:** The core avalanche operation relies on a 128-bit folded multiplication:
  ```rust
  let r = (a as u128).wrapping_mul(b as u128);
  ((r >> 64) as u64) ^ (r as u64)
  ```
  This single operation propagates bits across the entire 64-bit domain. It executes via fixed-width hardware multiplication, guaranteeing constant-time behavior.
- **Branchless Tail Handling:** Rather than looping or using a `match` statement for remaining trailing bytes, it uses overlapped 32-bit or 64-bit memory reads. For smaller chunks (1–3 bytes), it composites them mathematically without branching:
  ```rust
  let b0 = data[0] as u64;
  let b1 = data[len >> 1] as u64;
  let b2 = data[len - 1] as u64;
  a = (b0 << 16) | (b1 << 8) | b2;
  ```
- **Side-Channel Avoidance:** Control flow is determined solely by the *length* of the data slice, not its *contents*. By isolating the control structure to length-based blocks and applying fixed overlapping reads at the tail, `wyhash_64` entirely bypasses input-data-dependent branching.

## 3. `cityhash64`
**Location:** `crates/bcinr-logic/src/algorithms/cityhash64.rs`  
**Tier:** T0 — single-word arithmetic primitive

### Fixed Execution Width & Branchless Design:
- **Zero Control Flow:** The algorithm is a straight-line function composed strictly of fixed hardware instructions:
  ```rust
  let k0 = 0x9e3779b97f4a7c15;
  let x = val.wrapping_add(aux).wrapping_mul(k0);
  x ^ (x >> 33)
  ```
- **Side-Channel Avoidance:** As a `CC=1` primitive, it possesses no control flow graphs to trace. It guarantees a bounded `O(1)` execution cost and constant latency (sub-10ns), making timing attacks physically impossible at the algorithm level.
