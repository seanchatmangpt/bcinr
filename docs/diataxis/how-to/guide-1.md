# How to Verify a Function Compiles to Branchless Code

**Goal:** Confirm that a primitive contains no data-dependent branch instructions, so its execution path (and timing) is independent of input values.

**Prerequisites:** A release build toolchain and `cargo-show-asm` or `objdump`. Familiarity with the branchless primitives in [`mask.rs`](../../../crates/bcinr-logic/src/mask.rs). The compiler can re-introduce branches even when your source has no `if`, so trust the emitted assembly, not the source.

## Steps

1. Write the function with no control flow. Conditions become masks, never `if`:

   ```rust
   use bcinr_logic::mask::{lt_mask_u32, select_u32};

   /// Returns `hi` when `x < threshold`, else `lo` — no branch.
   #[inline(never)] // keep a standalone symbol so we can disassemble it
   pub fn pick(x: u32, threshold: u32, hi: u32, lo: u32) -> u32 {
       let m = lt_mask_u32(x, threshold); // 0xFFFF_FFFF or 0x0
       select_u32(m, hi, lo)
   }
   ```

2. Build in release mode so optimizations and target features are applied:

   ```bash
   cargo build --release -p bcinr-logic
   ```

3. Dump the assembly for just that symbol. With `cargo-show-asm` (`cargo install cargo-show-asm`):

   ```bash
   cargo asm --release -p bcinr-logic pick
   ```

   Or with `objdump` against the release `rlib`/binary:

   ```bash
   objdump -d --no-show-raw-insn target/release/libbcinr_logic.rlib | rg -A20 'pick'
   ```

4. Scan the listing for conditional **jumps**. On x86-64 these are `je`, `jne`, `jb`, `jbe`, `jg`, `jl`, etc. (anything starting `j` that is not the unconditional `jmp`). Their presence means the data path can diverge. Conditional *moves* (`cmov`), `setb`/`sete` + `neg`, and arithmetic are fine — they are constant-time.

## Verify it worked

- The disassembly for `pick` contains **zero** conditional jump instructions; you should see a `cmp`/`setb`/`neg` (mask construction) followed by `and`/`or` (selection), then `ret`.
- The cyclomatic-complexity gate agrees. Run it for the whole `algorithms/` tree:

  ```bash
  cargo make contract-gate
  ```

  Each public primitive must report complexity 1; any `if`/`match`/`for`/`while`/`loop` raises it above 1 and fails the gate.

See also: [Replace an if/else hot path with mask::select](./guide-2.md), [Guarantee WCET](./guarantee-wcet.md), [Run the cheat-scanner and contract-gate](./guide-7.md).
