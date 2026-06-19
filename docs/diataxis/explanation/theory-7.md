# WCET and Determinism

Worst-Case Execution Time (WCET) is the longest time a piece of code can
take over *all* admissible inputs and states. For hard real-time systems —
control loops, deadline-driven schedulers, safety interlocks — the WCET, not
the average, is the number that matters: missing a deadline is a fault even
if you usually finish early. This document explains why branchless
primitives are a good substrate for tight, trustworthy WCET bounds. The
practical recipes live in `docs/diataxis/how-to/guarantee-wcet.md`.

## Average time is the wrong question

Most performance work optimises the *mean*. Real-time analysis optimises the
*maximum*, and the two can diverge sharply. A branch that is predicted 99%
of the time has an excellent average and a worst case that includes the
mispredict flush; a cache that hits 99% of the time has an excellent average
and a worst case that includes a main-memory miss. WCET analysis must assume
the bad case happens, because over a long mission it eventually will.

```
   average-case thinking:  "it's ~5 ns"
   WCET thinking:          "it is NEVER more than N ns" (prove it)
```

## Determinism is what makes a WCET *tight*

You can always bound WCET pessimistically — assume every branch mispredicts,
every load misses. The trouble is that loose bounds force you to over-provision
the schedule. A *tight* bound requires the code's timing to be *predictable*,
and that is exactly what branchless primitives deliver.

A branchless primitive has a single execution path (`theory-1.md`), so its
instruction sequence is the same for every input. There is no
data-dependent branch to mispredict, so the misprediction term in the cost
model (`theory-2.md`) vanishes from the worst case. The WCET of the
primitive collapses to essentially its *only*-case time — best, average, and
worst coincide.

## The properties that buy a bound

Three structural features of the library map directly onto WCET reasoning:

- **Bounded, input-independent loops.** Where a branchless kernel iterates,
  it iterates a *fixed* or *length-determined* number of times, never a
  data-determined number. `bitonic_sort_8u32` (`network.rs`) executes a
  *sorting network* — a fixed schedule of compare-exchanges — so it performs
  the identical sequence of operations regardless of how unsorted the input
  is. `parse_hex_u32` (`parse.rs`) always runs its 8-iteration scan. The
  loop trip count is a constant or a function of length, both of which a WCET
  tool can resolve.
- **No hidden allocation.** The core is `no_std` with no allocator
  (`theory-9.md`), so there is no `malloc` whose latency depends on heap
  state, no GC pause, no page-fault-on-grow. Memory comes from a pre-sized
  arena (`mem.rs`) whose `alloc` is a branchless bounds check — constant
  time, never a syscall.
- **No panics in normal flow.** Operations that could fault instead return
  `Result` or saturate (`fix.rs`), so there is no unwinding path whose cost
  must be accounted for, and no abort that blows the deadline.

## Determinism is also reproducibility

WCET is one face of determinism; the other is that the *result*, not only
the timing, is identical across runs and platforms. Branchless arithmetic
over fixed-width integers is deterministic by construction — no
floating-point rounding modes, no iteration order that depends on
allocator-returned addresses, no thread-scheduling nondeterminism in the
single-path primitives. The SWAR and SIMD paths (`theory-4.md`) are held to
*bit-identical* results precisely so that a value computed on x86 matches the
one computed on ARM or WebAssembly. For consensus and replicated systems,
that reproducibility is as load-bearing as the latency bound.

## The boundary of the claim

Two honest qualifications. First, a *primitive* having flat timing does not
make a whole *program* real-time: preemption, interrupts, DMA, shared-bus
contention, and the memory hierarchy all contribute to system-level WCET,
and bcinr can only guarantee the part it owns. Second, the tightest bounds
still require checking the *compiled* code — an optimiser can reorder or
even reintroduce a branch — so WCET claims are validated against the emitted
instructions and against benchmarks (`cargo make bench`), not inferred from
source alone. What the library provides is the property that makes tight
bounds *attainable*: primitives whose worst case equals their only case.
