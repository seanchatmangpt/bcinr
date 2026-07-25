# BCINR API Layer: Bridging External Inputs to the Deterministic Substrate

The `bcinr-api` crate serves as a zero-cost, stable facade that pipes external data (e.g., untrusted network packets, UTF-8 strings) into the deterministic `bcinr-logic` substrate. It rigidly enforces the repository's rules (`CC=1`, `no_std`, zero-allocation) through its API design.

## 1. Zero-Cost, Pure Facade 
Exploration of `crates/bcinr-api/src/` reveals that the API avoids any abstraction penalties. The `src` directory acts exclusively as an exporter (e.g., `pub use bcinr_logic::dfa::{...}`). It curates 12 branchless domains (`mask`, `parse`, `scan`, `network`, `utf8`, `dfa`, `fix`, `int`, etc.) and maps them directly, meaning the API boundary itself compiles down to inline logic without adding any overhead or intermediate layers.

## 2. Maintaining the Zero-Allocation Boundary
The API layer prevents heap allocation in the hot path by dictating exactly how data crosses the boundary:
* **Borrowed Flat Slices over Objects:** Operations like `utf8::validate_utf8`, `scan::count_nonzero_bytes`, and `dfa::dfa_run` exclusively ingest borrowed `&[u8]` or `&[usize]` slices. No `Vec` or `String` buffers are accepted.
* **Fixed-Width Outputs:** Data extraction produces deterministic, fixed-size primitives. `parse::parse_decimal_u64` and `parse::parse_hex_u32` resolve string sequences strictly into primitive integers.
* **Feature-Gated Environments:** The `Cargo.toml` explicitly gates `alloc` and `std` behind optional features, defaulting to an environment where allocation functions don't exist. Memory is implicitly managed via patterns like `BumpArena` or `LockFreeSlab` (on the user side) while the API itself remains purely computational.

## 3. Branchless Translators
External inputs inherently pose semantic uncertainty. The API bridges this into a deterministic format using:
* **Mask Generation:** Functions exported in `scan.rs` and `mask.rs` (like `find_byte_mask`, `lt_mask_u32`, `select`) immediately translate data conditions into algebraic masks (e.g., `0xFFFFFFFF` or `0x0`). This maps external decisions straight into bitwise polynomials, satisfying the Radon Law ($CC=1$).
* **Flat State Machines:** Instead of parsing inputs using `match` blocks, `dfa::dfa_advance` uses a precomputed transition table (`table[state * alphabet_size + input]`) for state evaluation, allowing arbitrary external sequences to be digested branchlessly.
* **Sorting Networks:** Sorting untrusted external data uses unrolled sorting networks (e.g., `network::bitonic_sort_32u32`, `sort5_u32`) that perform constant-time, branch-free permutations.

Through these bounded inputs, fixed-width masks, and strict slice references, the API layer acts as a mechanical filter—ensuring the hot path stays structurally branchless and strictly allocation-free.
