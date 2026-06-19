# The Cost Model Behind Constant-Time Code

`theory-1.md` argued that data-dependent branches cost an unpredictable
pipeline flush. This document builds the *cost model* you should keep in
your head when reasoning about whether a branchless rewrite pays off. It is
deliberately a model, not a benchmark: real numbers come from
`docs/BENCHMARKS.md` and `cargo make bench`.

## Latency is not one number

Programmers often reason as if each operation has a single "cost." On a
pipelined, superscalar machine that is misleading. Three quantities matter,
and they differ:

- **Latency** — cycles from an instruction's inputs being ready to its
  result being ready. A 64-bit integer multiply might be ~3 cycles latency.
- **Throughput** — how many of that instruction can *start* per cycle. The
  same multiply might have a throughput of one per cycle even though its
  latency is three.
- **Dependency depth** — the length of the longest chain of operations that
  must happen in order. This, not the raw instruction count, sets the floor
  on how fast a kernel can run.

A branchless primitive trades a control dependency (the branch) for a
*data* dependency (the mask and the blend). The win is real only when the
new data-dependency chain is shorter, in expectation, than `branch +
occasional flush`.

## A worked comparison: `min_u32`

Consider the branching minimum versus bcinr's `min_u32`.

```
Branching:                      Branchless (mask.rs):
  cmp  a, b                       lt_mask_u32(a, b)  -> mask   (cmp; sbb/neg)
  jae  .else      <-- BRANCH      select_u32(mask,a,b)
  mov  r, a                         and  r1, mask, a
  jmp  .end                         andn r2, mask, b
.else:                              or   r, r1, r2
  mov  r, b
.end:
```

The branching version is *fewer* instructions and, when the branch is
perfectly predicted, *lower* latency. The branchless version is a fixed
short data-dependency chain (compare → mask → and/andn → or) with **no
branch**. The cost model says:

```
  E[branching]     = work + p_mispredict * flush_penalty
  cost[branchless] = fixed_chain_length          (no p_mispredict term)
```

When `p_mispredict` is near zero, the branch wins. As `p_mispredict` rises
(adversarial or random inputs), the `p * flush` term dominates and the
branchless version's *flat* cost becomes the better — and, crucially, the
*predictable* — choice.

## Constant time means constant *worst* case

The phrase "constant-time" in this library is a statement about the
*distribution* of latency, not just its mean. A branchless primitive
collapses that distribution to (nearly) a point: every input drives the
same instructions in the same order, so the latency histogram is a spike,
not a long tail.

```
  Branching kernel latency:        Branchless kernel latency:
     |#                                  |
     |#                                  |
     |##         . (mispredict tail)     |     #
     |####  .  .   .                     |     #
     +-------------------- ns            +-------------------- ns
```

The flat distribution is what makes the downstream guarantees possible:
worst-case execution time (`theory-7.md`) and side-channel resistance
(`theory-6.md`) are both properties of the *tail*, and branchless code has
essentially no tail.

## Second-order costs the model must include

A faithful cost model does not stop at the ALU:

- **Code size / I-cache.** Computing both arms can enlarge the hot path.
  Usually negligible, occasionally not.
- **Register pressure.** Holding both candidate results uses more
  registers; under pressure this can cause spills that erase the win.
- **Memory.** Table-driven branchless designs (see `dfa.rs`) move the
  decision into a lookup, trading a branch for a load — fast if the table
  is cache-resident, a hazard if it is not (`theory-3.md`).

The honest summary: branchless code buys *predictability* and pays in
*throughput*. The cost model is the tool for deciding when that exchange is
favorable, and `theory-10.md` turns it into concrete guidance.
