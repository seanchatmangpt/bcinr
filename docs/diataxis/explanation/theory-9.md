# The no_std, Zero-Dependency Philosophy

The core of bcinr — `crates/bcinr-logic` — compiles without the Rust
standard library, without an allocator, and with no external runtime
dependencies. This is a deliberate constraint, not an oversight, and it is
load-bearing for every other guarantee the library makes. This document
explains *why* the constraint exists and what it costs.

## What `no_std` actually removes

`#![no_std]` drops the standard library and keeps only `core` (and, when an
allocator is present, `alloc`). Concretely, the core layer gives up:

- the heap — no `Box`, `Vec`, `String`, `HashMap` without explicitly opting
  into `alloc`;
- `std::io`, threads, time, filesystem, and the OS abstraction layer;
- panic *infrastructure* that assumes an unwinder and a process to abort.

What remains is enough to do real work: fixed-width integers, slices,
`Option`/`Result`, `core::mem`, and the arithmetic and bitwise operators
that branchless code is built from. Every primitive in `mask.rs`, `int.rs`,
`bitset.rs`, `fix.rs`, and the rest is expressible in that vocabulary —
which is the point.

## Why the determinism story requires it

`no_std` is not austerity for its own sake; it is what makes the WCET and
determinism arguments of `theory-7.md` *true* rather than aspirational. The
standard library is full of operations whose latency depends on global,
hidden state:

- `malloc`/`free` latency depends on heap fragmentation and may call into
  the OS;
- collection growth (`Vec::push` past capacity) hides an allocation and a
  copy;
- a garbage collector or even a destructor avalanche introduces pauses you
  did not write.

Remove the allocator and those nondeterminism sources cannot occur.
Memory in bcinr comes from a caller-sized arena (`mem.rs`) whose `alloc` is
a *branchless bounds check* — constant time, no syscall, no fragmentation.
There is no allocation on the hot path to perturb the timing distribution.

## Why the safety story requires it

The library's memory-safety claim rests on `#![forbid(unsafe_code)]` across
every algorithm module, with exactly three audited exceptions documented in
`SAFETY.md`. Staying in `core` keeps that boundary small and checkable:
fewer dependencies means less code that *could* contain `unsafe`, less
supply-chain surface, and a smaller audit. The `CLAUDE.md` policy — "zero
runtime dependencies in the security-critical layer" — is a direct
consequence: every dependency you do not have is a dependency you do not have
to trust, audit, or pin.

## Where this lets bcinr run

A `no_std`, allocator-free core runs in environments where `std` cannot:

```
   bare-metal firmware / MCUs   (no OS, no heap)
   kernel and driver context    (no std runtime)
   WebAssembly                   (portable, sandboxed, no syscalls)
   secure enclaves / defence-in-depth   (minimal trusted computing base)
```

Portability falls out of the same constraint. Because the core leans on SWAR
over plain integers (`theory-4.md`) rather than platform intrinsics, the
identical source compiles for x86-64, ARM, and WebAssembly, with SIMD as an
*optional acceleration* layered on top — never a requirement.

## The cost, paid honestly

The discipline is not free. Without the heap you size buffers up front and
work within fixed capacities; without `std` collections you reach for
arena-backed or fixed-size structures; some conveniences simply are not
available in the core and live in the higher facade crates instead. This is
a real ergonomic tax. The trade is intentional: the core pays in convenience
to buy *determinism, a minimal trusted computing base, and portability*, and
the layered workspace (`bcinr-core`, `crates/bcinr-api`) exists precisely so
that applications which *can* use `std` get ergonomics without forcing those
costs back down into the verified substrate.
