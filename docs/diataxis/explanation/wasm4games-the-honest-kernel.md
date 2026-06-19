# wasm4games: The Honest Kernel

> Explanation — the epistemics of `crates/wasm4games`. Not *what* a kernel computes
> (that is the [pattern catalog](../reference/wasm4games-patterns.md)) and not *how you add
> one* (that is the [tutorial](../tutorials/wasm4games-add-a-pattern.md)), but the harder
> question: **how do you know it is right, that the test is not lying, and that what I ran
> is what you ran?** For the shape of the crate, read the
> [foundry overview](wasm4games-overview.md) first.

## The three questions

A pattern kernel makes a strong claim: it is a deterministic, branchless, auditable
implementation of a game behavior. Three doubts sit under that claim, and an honest kernel
has to answer each one without hand-waving:

1. **How do I know it's right?** — that the optimized branchless body actually computes the
   specified behavior, and not a plausible near-miss.
2. **How do I know the test isn't lying?** — that a passing test suite reflects a correct
   kernel rather than a test too weak to fail.
3. **How do I know what I ran is what you ran?** — that the kernel I execute here produces
   the same result you execute elsewhere, in another language or on another target.

`wasm4games` answers these with three concrete, in-tree mechanisms: the **reference
oracle**, the **two-sided corpus with a weakened-variant falsifier**, and the **golden
digest with a C-ABI receipt**. None of them is a slogan; each is code that fails loudly
when the claim is false. The doctrine line frames where the kernel's authority ends:

> Engines project worlds; wasm4games operates patterns; wasm4pm admits evidence; ggen
> manufactures the law.

The honest kernel proves the *operates patterns* leg. It does **not** claim the *admits
evidence* leg — that is fenced explicitly below.

## 1. "How do I know it's right?" — the reference oracle

Every kernel ships beside a deliberately **branchful** twin, `<name>_reference`. The
branchless body is the optimized form (masks, saturating arithmetic, `lt_mask_u32`); the
oracle is the obvious, readable form with ordinary `if`s. They are two independent
encodings of the *same* authority — the one-line legality or transition spec stated in the
kernel's doc comment.

A `proptest` named `equivalence` then asserts they agree across randomized inputs:

```rust
proptest! {
    #[test]
    fn equivalence(s in any::<u64>(), i in any::<u64>()) {
        prop_assert_eq!(
            movement_legality_checked_reference(s, i),  // readable, branchful
            movement_legality_checked(s, i)             // optimized, branchless
        );
    }
}
```

This is the answer to "is it right": the fast version is pinned to a slow version a human
can read and check against the spec. A bug in the branchless arithmetic that diverges from
the oracle on *any* sampled input fails the build. The branchless body never gets to be its
own judge.

Two `// Hoare-logic Verification Line N: …` annotations sit at the foot of each kernel,
stating the pre/post condition in prose (e.g. "verdict bit0 is set iff |proposed − current|
exceeds max_speed"; "a displacement equal to max_speed is admitted"). They name the
invariant the oracle and the boundary tests are jointly defending.

## 2. "How do I know the test isn't lying?" — two-sided corpus + weakened teeth

An equivalence test can still be vacuous: if the oracle itself is wrong, the kernel can
match it and both can be wrong together; and a corpus that only contains *good* inputs
cannot distinguish a real detector from one that rubber-stamps everything. `wasm4games`
attacks both failure modes by **falsifying the test, not just the kernel**.

### Value-mutants make the equivalence test sharp

Three mutants — `cf1`/`cf2`/`cf3` — are deliberately wrong variants of the oracle (negate,
off-by-one, bit-flip). Each `cf` proptest asserts the *correct* answer **differs** from the
mutant:

```rust
fn mutant_2(s: u64, i: u64) -> u64 {
    movement_legality_checked_reference(s, i).wrapping_add(1)  // off by one
}
// ...
fn cf2(s in any::<u64>(), i in any::<u64>()) {
    let r = movement_legality_checked_reference(s, i);
    prop_assert!(r != mutant_2(s, i));   // the suite must be able to SEE this error
}
```

A test suite that cannot tell the right answer from an off-by-one is not testing anything.
The mutants prove the suite has the resolution to *catch* a wrong answer — they are the
test's self-check.

### The two-sided corpus gives the behavior teeth

For detector kernels especially, the behavior is judged against a **two-sided fixture
corpus**: `must_admit` (legal cases the kernel must accept) and `must_refuse` (cheat cases
it must reject).

```rust
fn must_admit()  -> &'static [(u64, u64)] { /* no movement; exactly at the limit; ... */ }
fn must_refuse() -> &'static [(u64, u64)] { /* forward teleport; backward teleport; ... */ }
```

The refuse side is the load-bearing half. A kernel that always returns "admitted" passes
any all-good corpus; only `must_refuse` can catch it.

### The weakened variant proves the corpus itself has teeth

The deepest move is to falsify the *corpus*. Each kernel includes a `weakened` twin — a
deliberately broken implementation (the teleport check deleted, so every move is admitted) —
and a test that the weakened detector **admits at least one case from the refuse corpus**:

```rust
fn weakened(s: u64, i: u64) -> u64 { let _ = (s, i); 0 }  // check removed: admit everything

#[test]
fn weakened_fails_corpus() {
    assert!(
        must_refuse().iter().any(|&(s, i)| weakened(s, i) == 0),
        "a weakened detector must admit >=1 cheat from the corpus"
    );
}
```

Read it as a contrapositive: *if removing the check were undetectable, the corpus would be
proving nothing.* The test demands that removing the check **is** detectable. This is what
separates "the test passes" from "the test would have failed if the kernel were wrong" — the
property that makes a green suite mean something.

## 3. "How do I know what I ran is what you ran?" — golden digest + C-ABI receipt

Correctness on this machine is not portability. The third mechanism turns "same input →
same output" into a single number that any target must reproduce.

The whole catalog folds into `corpus_digest()`: for every pattern it records the id, event
code, OTEL span, object codes, and admission rule, **and** the kernel's output over four
fixed probe vectors (zero, saturated, a mid-range case, an arbitrary case), all into one
rolling FNV-1a receipt. That value is frozen as `GOLDEN_CORPUS_DIGEST` (pinned in
`crates/wasm4games/src/corpus.rs`). Any drift — a kernel, the registry, or the evidence
wiring — changes the digest and fails loudly.

The digest becomes a **cross-language receipt** through `wasm4games-capi`, a C-ABI
`staticlib` exposing `w4g_corpus_digest()` and `w4g_golden_corpus_digest()`. A C harness
links the same kernels the Rust tests exercise and compares:

```text
   one ontology ─ggen→ committed kernels ─┬─ Rust:  corpus_digest() == GOLDEN_CORPUS_DIGEST
                                          └─ C ABI: w4g_corpus_digest() == w4g_golden_corpus_digest()
                            both sides reproduce the SAME number  ⇒  C-ABI portability receipt
```

When both sides reproduce the same number, "what I ran is what you ran" is no longer a
promise — it is a checked equality across two languages from one source. That is the
executable form of the falsifier *same input → same output → same receipt across targets.*

## What this argument does NOT cover (the fences)

An honest kernel is also honest about the edge of its evidence. The three mechanisms above
prove a bounded claim; everything past that boundary is fenced, not assumed:

- **External admission is not proven here.** wasm4games *operates patterns* and emits
  candidate evidence; it does **not** admit it. Real admit/refuse verdicts come only from
  the external **wasm4pm** authority via the workspace-excluded `wasm4games-wasm4pm` bridge,
  which is online-only. **wasm4pm is referenced, not vendored.** The in-crate `verify`
  module is an explicit *offline proxy* (bounds-checks status codes, confirms replay
  determinism, and *constructs* negative fixtures a real authority ought to refuse) — it is
  not the authority. So: **wasm4games = `VERIFIED_UNDER_SCOPE`**, and the conformance leg
  is external and unproven offline.
- **Timing is unmeasured.** Kernels are branchless *by construction* and carry a prose
  timing contract, and a `bench` module exists — but it is dormant (behind
  `#[cfg(feature = "bench")]`) and no latency number is claimed here. "Branchless" is a
  structural property; it is **not** a measured wall-clock guarantee in this tree.
- **Portability is verified only for the C ABI leg.** **C-ABI portability is VERIFIED**
  (Rust native + C reproduce the one digest). **wasm32 is UNVERIFIED** and **engines / Lua
  are NOT_STARTED** — the digest has not yet been reproduced under a wasm runtime or an
  engine adapter.
- **Byte-stable regeneration is asserted, not receipted.** **`GATE-W4G-001`** (run
  `ggen sync` and prove the committed kernels regenerate byte-for-byte) is **UNVERIFIED**
  because `ggen` is not installed in this environment. The covenant requires regeneration;
  the gate that would *receipt* it is still open.
- **The workspace crown is blocked elsewhere.** Independently of wasm4games,
  **`bcinr workspace crown = BLOCKED_BY_PRE_EXISTING_CONTRACT_GATE_RESIDUAL`** — a
  pre-existing residual unrelated to this crate's scope.

## Why this is "the honest kernel"

Put together, the three mechanisms answer the three doubts in kind:

| Doubt | Mechanism | What fails if the claim is false |
|---|---|---|
| Is it right? | branchless body **vs** `_reference` oracle, checked by `equivalence` | proptest finds a diverging input |
| Is the test lying? | value-mutants + two-sided corpus + `weakened` falsifier | a wrong kernel, or a toothless corpus, is *observable* |
| Same run for you and me? | golden digest reproduced over the C ABI | the two-language digest comparison is unequal |

The honesty is not that the kernel is proven correct in some absolute sense — it is that
**every claim is paired with the thing that would expose it if it were false**, and the
claims that *aren't* yet backed (external admission, measured timing, wasm32/engine
portability, byte-stable regen) are named as fences rather than quietly implied. An honest
kernel is one whose green test suite you can trust *because you can see exactly what would
have turned it red* — and whose scope you can trust *because it tells you where it stops*.
