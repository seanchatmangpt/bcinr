# Branch Misprediction and the Pipeline Stall

This document explains *why* a conditional branch can be expensive, and why
that expense is the root motivation for everything else in bcinr. It is
understanding-oriented: if you want signatures, see the reference section.

## The pipeline is a conveyor belt

A modern out-of-order CPU does not execute one instruction at a time. It
keeps a *pipeline* of instructions in flight — fetch, decode, rename,
schedule, execute, retire — so that while one instruction is executing,
the next dozen are already partway through the machine. On a contemporary
x86-64 core the pipeline is roughly 14–20 stages deep, and the core can
have well over a hundred instructions in flight at once.

This works only if the CPU knows *which* instructions come next. For
straight-line code it always does. For a conditional branch it does not:
the direction of the branch depends on a value that may not be computed
yet.

## Speculation, and the cost of being wrong

Rather than stall and wait, the CPU *guesses*. The branch predictor — a
collection of history tables and pattern matchers — predicts the direction
and target of the branch and begins speculatively executing down the
predicted path. If the guess is right, nothing was lost. If the guess is
wrong, every speculatively-executed instruction must be squashed, the
pipeline flushed, and fetch restarted from the correct address.

That flush is the *misprediction penalty*. Its size is essentially the
depth of the pipeline: the work that was in flight is thrown away. In
round numbers, a single mispredict costs on the order of **15–20 cycles**.

```
  Correct prediction:   F D R S E ... retire      (no bubble)
  Misprediction:        F D R S E  <-- squashed
                        ......... pipeline refill .........  (15-20 cyc bubble)
```

## Why "rare" is not "free"

The predictor is good — frequently 95%+ accurate on typical code. The
trap is that *average* accuracy hides *worst-case* latency. Consider a
branch that depends on input data with no exploitable pattern, e.g. a sign
test on adversarial or cryptographic input. The predictor degrades toward a
coin flip, and you pay the penalty on roughly half of all executions.

For throughput-oriented batch code this may average out. For the workloads
bcinr targets — latency-critical, deterministic, side-channel-sensitive —
it does not. A consensus loop, a real-time control step, or a constant-time
comparison cannot afford a 15-cycle bubble whose *occurrence* depends on the
secret or on the adversary's input.

## The branchless answer

A branchless primitive removes the data-dependent branch entirely. Instead
of *choosing* a path, it computes *both* possible results (or an arithmetic
blend of them) and selects with a mask:

```rust
// Instead of:  if a < b { a } else { b }
// bcinr computes a mask and blends — no branch to mispredict.
let mask = lt_mask_u32(a, b);   // 0xFFFF_FFFF or 0x0
select_u32(mask, a, b)          // (mask & a) | (!mask & b)
```

There is now exactly one path through the code. The predictor has nothing
to guess; there is no bubble to pay; the latency is the same on every input.
This is the structural property the rest of the library calls *time
invariance*, and it is the foundation of the constant-time, WCET, and
side-channel guarantees discussed in the sibling explanation documents.

## What this does *not* claim

Removing a branch is not automatically a win. The blended form does strictly
more arithmetic than the taken path of a well-predicted branch, so on
predictable inputs branchless code can be *slower*. The point is not that
branchless is always faster; it is that branchless is *predictable*. When
the cost of an occasional flush — or the information leaked by its timing —
is worse than a few extra ALU ops on every call, you trade the branch away.
The trade-offs are examined in detail in `theory-10.md`.
