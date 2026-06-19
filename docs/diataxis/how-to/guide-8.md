# How to Build the Core for a `no_std` / WebAssembly Target

**Goal:** Compile `bcinr-logic` for a bare-metal or WebAssembly target with no `std` and no allocator, confirming the logic layer is genuinely freestanding.

**Prerequisites:** `bcinr-logic` is `#![no_std]` (see the crate root, [`lib.rs`](../../../crates/bcinr-logic/src/lib.rs)). Its features are `alloc` and `std`, both **off by default**. You need the target installed via `rustup`.

## Steps

1. Confirm the default build pulls in no `std`. Build the crate with default features only:

   ```bash
   cargo build -p bcinr-logic --release
   ```

   The crate root declares `#![no_std]` and only links `alloc`/`std` behind their cargo features, so the default build is freestanding.

2. Add a target. For WebAssembly:

   ```bash
   rustup target add wasm32-unknown-unknown
   ```

3. Cross-compile for that target with **no default features**, so neither `std` nor `alloc` is enabled:

   ```bash
   cargo build -p bcinr-logic --target wasm32-unknown-unknown --no-default-features
   ```

   This will fail to compile if any code path reached `std` or the allocator unconditionally — which is exactly the check you want.

4. If you do need heap collections (`Vec`, `Box`) on the target, opt in explicitly rather than pulling in `std`:

   ```bash
   cargo build -p bcinr-logic --target wasm32-unknown-unknown \
       --no-default-features --features alloc
   ```

5. Use the primitives from a downstream `no_std` crate by importing from the modules directly; they take and return plain integers, so nothing requires an allocator:

   ```rust
   #![no_std]
   use bcinr_logic::int::popcount_u64;
   use bcinr_logic::mask::select_u64;

   pub fn weight(x: u64, fallback: u64) -> u64 {
       select_u64(0u64.wrapping_sub((x != 0) as u64), popcount_u64(x), fallback)
   }
   ```

## Verify it worked

- The `--no-default-features` cross-build succeeds for `wasm32-unknown-unknown`, proving zero `std`/allocator dependency in the default surface.
- A `.wasm` artifact appears under `target/wasm32-unknown-unknown/release/`.
- The whole workspace still passes its normal checks on the host:

  ```bash
  cargo make check
  ```

See also: [Choose between saturating and wrapping arithmetic](./guide-4.md), [Guarantee WCET](./guarantee-wcet.md).
