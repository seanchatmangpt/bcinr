# The Bounded Substrate Manifesto

**bcinr / BRCE — June 2026, draft**

---

## 1. The claim

> AGI programs built as open-ended software systems cannot prove the nontrivial
> behavioral properties they need for safety, coherence, and alignment, because
> those properties fall under Rice's Theorem — unless the system is restricted
> into a bounded, decidable fragment for the questions that actually matter.

This is not a claim that intelligence is bounded. It is a claim about where the
*authority* to act has to live if you want that action to be provable rather
than merely plausible.

We do not build a smarter planner. We build a substrate that makes a narrower
set of questions — *is this admissible, is this schedule legal, did this
execution happen the way it claims* — answerable exactly, every time, in
bounded time, instead of approximately, sometimes, by simulation or sampling.

---

## 2. The exclusion stack

Most "AGI program" efforts inherit a failure mode before they write a line of
code, by inheriting an organization before they have a working system:

```text
AGI program
  ↓
complex organization
  ↓
complex language/runtime/framework stack
  ↓
unbounded semantic behavior
  ↓
Rice boundary
```

Walking down that stack:

- **Gall's Law.** Complex systems that work evolve from simple systems that
  worked. Most AGI efforts start from a complex *organization*, not a simple
  working ancestor — they assemble the org chart before the working core
  exists.
- **Conway's Law.** The resulting system inherits the organization's
  communication structure — and its communication failures. Planning,
  scheduling, policy, and audit get built by different teams, in different
  languages, with different assumptions, and the seams between them are where
  correctness leaks out.
- **Little's Law.** Queueing math doesn't care about intent. Add more agents,
  more teams, more review gates, more safety layers, and queues grow with
  them. Latency and backlog are not solved by adding more checkpoints; they
  are *caused* by it past a point.
- **Chesterton's Fence.** Under that load, inherited constraints get removed
  to relieve pressure — without anyone proving what the constraint was
  actually for. Bounds erode first; that's exactly the move that re-opens the
  Rice boundary.
- **Framework/language complexity.** The implementation substrate is already
  semantically enormous before any of this starts: dynamic runtimes,
  distributed orchestration, opaque transitive dependencies, nondeterministic
  async scheduling, GPU kernels, eval harnesses, tool-calling layers. Each one
  is, on its own, a source of unbounded behavior.
- **Rice's Theorem.** Once the system is just "an arbitrary program," every
  nontrivial semantic property of its behavior — does it terminate, does it
  obey this invariant, will it ever do X — is undecidable in general. Not
  hard. Undecidable. No amount of testing, evaluation, or scale changes that;
  it only changes your confidence interval.

The honest conclusion most AGI programs don't draw: you cannot out-engineer
Rice's Theorem by being more careful inside an unbounded system. You have to
stop asking Rice-forbidden questions about the unbounded part, and ask
provable questions about a bounded part instead.

---

## 3. The move

```text
unbounded program semantics   →  undecidable
bounded law-state substrate   →  decidable, locally
```

They are trying to verify "general intelligence" over arbitrary computation.
That is Rice territory by construction. Our move is not a clever trick around
the theorem — there is no trick. It is a refusal to put arbitrary computation
on the authority surface at all.

The model — the LLM, the planner, the proposer, whatever you call the part
that's actually doing open-ended reasoning — **may propose**. It is never
trusted. It is always checked, by a substrate that was built small enough on
purpose to make checking exact rather than approximate:

```text
POWL DAG        — structural law:    what can legally happen concurrently
PDDL8 plan      — temporal geometry: when, how long, under what resources
Prolog8 proof   — admission calculus: is this specific action permitted
BLAKE3 receipt  — execution identity: did this happen, exactly, and can it replay
```

The proposer is unbounded. The authority surface is not. That asymmetry is the
entire architecture.

---

## 4. Why bounded, specifically — the "8" discipline

The bound isn't an implementation limitation that we'll lift once hardware
improves. It's a load-bearing design constraint, threaded through the whole
stack: bounded arity, bounded body length, bounded recursion depth, a 64-op
tape ceiling, fixed-size masks instead of arbitrary graphs.

Bounded systems convert questions from heuristic to algorithmic:

```text
Is this schedule legal?            → graph check over a bounded DAG
Which resources bind?              → finite-difference replan, bounded count
Which actions are equivalent?      → linearization check over a bounded DAG
Can these receipts replay?         → pure function, deterministic, bounded steps
Does this policy ever admit a
  forbidden pair, across the full
  reachable state space?           → exhaustive, not sampled, because the
                                      ground action/fluent space is finite
```

None of these are smarter than an unbounded planner. All of them are *exact*
in a way an unbounded planner's answer to the same question cannot be, because
the unbounded version's answer is, in the general case, Rice-forbidden.

We've already built and tested several of these exactly:

- `find_temporal_plan` schedules every applicable durative action per tick
  (not just the first), so concurrency emerges from capacity constraints
  rather than being assumed.
- `ScheduleAnalysis64` computes critical path, slack, max parallelism, binding
  resources, and capacity ±1 sensitivity as a bounded graph computation over
  the existing `pred_mask`/`succ_mask` DAG and finite-difference replans — no
  new solver, no LP, no polytope machinery, because none of that was needed
  to answer these specific bounded questions.
- Prolog8 deliberately restricts itself to a decidable, terminating,
  side-effect-free, positive-Horn fragment (fixed byte caps, no cut, no
  unbounded recursion, `ProofMode::PositiveOnly`) — every one of those
  restrictions exists *because* it's load-bearing for the admission gate, not
  in spite of being a limitation. Absence of a fact means Deny by default,
  not "unknown, proceed."
- A BLAKE3 receipt chain is a pure function of the executed steps — replaying
  it is recomputation, not trust.

---

## 5. What this is not

This is not a claim that the substrate makes the *system* intelligent, safe,
or aligned in some emergent, automatic sense. It does not make planning
optimal — `find_temporal_plan` finds *a* feasible schedule, not provably the
best one, and we say so. It does not compute the full feasible-region
boundary or do sensitivity analysis over an optimal scheduler — that's
explicitly deferred, scoped, and listed as future work, not silently assumed.

It is also not a claim that composing bounded units makes the *composed*
system's global analysis free. Verification work can be localized to bounded
units and their interfaces — that is a real and valuable systems property —
but it does not automatically mean the whole assembled system stays O(1) as
the number of components grows. Global questions about a large composition
can still grow with the composition's size. What we get is the ability to
*localize* verification to bounded units instead of re-verifying an
unstructured global state from scratch every time, which is a different and
more modest claim than "infinite scale, zero cost."

And it is not a route around Rice's Theorem in the sense of solving the
general case. It is a refusal to ask the general-case question in the part of
the system where an action actually gets to happen.

---

## 6. The wager

The long-term hypothesis: bounded, independently verified systems can be
composed into increasingly capable execution substrates, letting a relatively
modest proposer — an LLM, or anything else willing to propose rather than
unilaterally act — operate inside an environment where correctness, policy,
resource feasibility, and replay are enforced by the substrate, not entrusted
to the model. Generality, if it appears, emerges from the substrate's
composability under load, not from the proposer getting smarter.

That is a testable architectural hypothesis, not a belief. Every claim in this
document maps to something either already implemented and tested in this
repository, or explicitly marked as future work. We intend to keep it that
way: nothing in this manifesto should describe a capability that isn't either
real or labeled as not yet real.

---

*This is a working draft. It should be revised as the substrate grows, and any
claim here that stops matching the code should be corrected or removed, not
left to drift.*
