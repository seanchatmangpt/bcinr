# Trade-offs: When Branchless Is *Not* Worth It

Every other document here argues *for* branchless code. This one argues
against it — or rather, it draws the boundary. Branchlessness is a tool with
a cost (`theory-2.md`), and applied indiscriminately it makes code slower,
larger, and harder to read for no benefit. A library that takes its own
performance claims seriously has to be honest about when its central
technique is the wrong call.

## The core trade, restated

A branch *skips* work; a branchless rewrite *does both arms and selects*. So
the exchange is always the same:

```
   branch:      cheap when predicted, expensive (flush) when not, variable timing
   branchless:  fixed cost (often higher), no flush, flat timing
```

Branchless wins when the flush term or the timing variance is what hurts.
Branchless *loses* when the branch is cheap because it is predictable and you
do not care about the variance. Recognising which world you are in is the
whole skill.

## When to keep the branch

**1. The branch is highly predictable.** Loop guards, `is_empty()` checks,
error paths taken ~0% of the time, configuration read once at startup — the
predictor nails these, the flush term is ~0, and a branchless rewrite just
adds arithmetic to the common path. Leave them alone.

**2. The arms are expensive or have side effects.** Computing *both* sides
is only acceptable when both are cheap and pure. If one arm does I/O,
allocates, or runs an expensive sub-computation, you must not execute it
speculatively just to discard it. "Compute both, select one" assumes the
discarded work was free; when it is not, branch.

**3. The condition is not on the hot path.** Branchlessness pays where the
code runs millions of times under latency or timing pressure. In cold setup
code, an admin endpoint, or anything off the critical loop, clarity beats a
micro-optimisation that will never show up in a profile.

**4. The compiler already did it.** Modern backends emit `cmov`, `SETcc`,
predicated instructions, and vectorised selects from ordinary `if`. Often the
clearest source *is* the branchless machine code. Check the emitted assembly
before hand-rolling bit tricks; if the compiler already removed the branch,
a manual rewrite only costs readability.

## When the data-dependent address is the real problem

A subtle case: removing the branch does not help if the leak or the stall is
through *memory*, not control flow (`theory-6.md`). A branchless function
that indexes a table with a secret still leaks via the cache, and a
branchless kernel that chases data-dependent pointers still stalls on
misses. Branchlessness addresses the control-flow channel; if your problem
is the memory channel, branchless-ness alone buys nothing — you need a
data-independent *access pattern*, which is a different (and sometimes
costlier) design.

## The readability tax is a real cost

Branchless code converts intent into arithmetic. `select_u32(mask, a, b)`
needs the mask convention to be understood; the `eq_mask` MSB trick is not
self-evident; a sorting network hides the comparison structure that an
`if`-based sort makes obvious. This opacity has a price: more reviewer time,
more places to hide a bug, and a higher chance that a "clever" rewrite is
subtly wrong. The project's own anti-patterns (`anti-patterns.md`) — vacuous
XOR identities, magic constants, padding — are partly symptoms of reaching
for branchless tricks where they add nothing. The discipline is to pay the
readability tax *only* where the predictability or side-channel benefit
justifies it.

## A decision checklist

Reach for a branchless primitive when **any** of these holds:

- the condition depends on adversarial, random, or secret data (unpredictable
  branch, or a timing channel);
- the code is on a hot, latency-critical, or hard-real-time path where the
  worst case is what matters (`theory-7.md`);
- you need constant-time behaviour for side-channel reasons (`theory-6.md`);
- you need bit-identical, deterministic results across platforms
  (`theory-9.md`).

Keep the branch when **all** of these hold:

- the branch is well-predicted in practice;
- the arms are cheap and free of side effects, *or* one arm is expensive;
- the code is cold or off the critical path;
- the branching version is clearer and the compiler is not already removing
  the branch for you.

Branchless programming is a precision instrument, not a style. Used where the
cost model (`theory-2.md`) says it pays, it buys determinism, WCET
tightness, and side-channel resistance. Used everywhere, it buys slower,
murkier code. Knowing the difference is the point of this entire explanation
series.
