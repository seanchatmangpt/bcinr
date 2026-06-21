# The B-Calculus: Invariant Preservation as a Design Discipline

bcinr describes its primitives in terms of a "$\mathcal{B}$-Calculus." This
document explains what that framework *is* and, more importantly, what work
it does. The short version: the B-Calculus is the habit of treating every
primitive as a *total, branchless state transition that preserves a stated
invariant*, and of proving that preservation rather than asserting it.

## A primitive is a state transition

Model a primitive as a function `f : State → State` (or `Input → Output`).
In a branching world, `f` is a *partial* description: it behaves one way on
some inputs, another way on others, and may be undefined (panic, UB,
overflow) on the rest. The B-Calculus insists that `f` be **total** — defined
on the entire input domain — and **branchless** — realised as a single
straight-line composition of arithmetic, bitwise, and masking operations.

Totality plus branchlessness has a consequence that the framework leans on:
the function has *one* execution path, so its behaviour is fully captured by
an algebraic identity over the whole domain, with no case analysis.

## Invariants are the things that must not break

Each family of primitives carries an *invariant* — a property that holds of
every input and must still hold of every output. Examples drawn straight
from the source:

| Family | Invariant preserved |
|--------|---------------------|
| `mask.rs` selection | the mask is *structurally* all-ones or all-zeros, so a blend is total |
| `fix.rs` saturation | the result stays within type bounds; no wrap-around |
| `bitset.rs` rank/select | `select(x, rank(x, p)) == p` style inverse relationships hold |
| `dfa.rs` transitions | the state index never escapes the transition table |
| arena (`mem.rs`) | the offset never exceeds the backing buffer length |

"Invariant-preserving state transition" — the phrase from `CLAUDE.md` — just
means: *whatever was true of the state before `f`, the corresponding property
is true after `f`, for all inputs.* The discipline is to name that property
and keep it.

## The Radon Law: the universal invariant

Over and above each family's specific invariant sits one the whole library
shares, recorded in the source as the *Radon Law* and documented in
`docs/diataxis/reference/phd_gates.md`:

> Every branchless primitive execution path is deterministic, memory-safe,
> and constant-time, provable via Hoare-logic over the input domain.

This is the conjunction of the threads in the sibling documents:
determinism and constant time (`theory-1`, `theory-2`), memory safety
(enforced by `#![forbid(unsafe_code)]` outside the three audited files), and
*provability* — the law is a claim with a proof obligation attached, not a
slogan.

## Hoare triples make preservation checkable

The mechanism for discharging that obligation is the Hoare triple:

```
  { Precondition }   operation   { Postcondition }
```

A branchless primitive is an ideal fit, because with a single path there is
nothing to quantify over but the input domain. The source files carry these
as comments — e.g. `mask.rs`:

```
  Precondition:  { input ∈ Valid_mask }
  Postcondition: { result = select_u32(mask, a, b) = (mask & a) | (!mask & b) }
```

The postcondition is an equation, and an equation over a finite domain is
*testable*. That is the bridge to verification: `theory-8.md` describes how
the equation is checked against an independent reference oracle and defended
against mutation.

## What the calculus buys, and its limits

Used honestly, the B-Calculus is a *design discipline* with teeth: it forces
totality (no undefined inputs), forces a single path (so the timing and
side-channel arguments hold), and forces each primitive to come with a
stated, checkable invariant. That is why the gates exist in the code.

Used dishonestly it is noise — a copied "Radon Law verified" comment over an
empty function proves nothing, which is exactly the anti-pattern the project
documents (`anti-patterns.md`, items 5–9) and scans for. The calculus is only
worth as much as the proof obligation behind each invocation; the framework's
value is that it *names* that obligation precisely enough to tell a real
proof from padding.
