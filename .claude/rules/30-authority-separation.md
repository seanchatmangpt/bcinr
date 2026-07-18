# Authority Separation — SELECT Is Never DO

Generalizes AGENTS.md §4 (constitutional roles) and the CMCA C2/C3 authority model into an
unconditional rule applied on every touch of authority-bearing code in this repo, regardless of
crate, module, or feature branch.

## The Law

SELECT (choosing among admitted alternatives — observing, measuring, scoring, or proposing) is
never DO (actuating a real-world or persistent-state effect). A function whose contract is
selection or observation must not itself:

- mutate persistent state (files, databases, global/static state, external systems), or
- mint a certificate, admitted-state token, or sealed learning/switch token.

Authority-bearing types — any type a downstream system trusts as proof some check already
passed (certificates, admitted states, sealed learning/switch tokens) — must be opaque:

- no public raw fields exposing the payload for direct construction,
- no safe-Rust construction path reachable from outside the type's own crate-internal admission
  function,
- no derive (`Default`, `Deserialize`, `From`, or equivalent) that opens an alternate
  construction path. A derive that lets `T::default()` or `serde` deserialization produce a
  value indistinguishable from one the admission function issued is itself a second
  constructor and violates this law.

**Cardinality law:** for any authority type representing that an adaptive mutation was
certified, there must be exactly one production constructor path. This is proved by counting
every reachable public/pub(crate) construction site across the crate boundary — not asserted by
description, not inferred from the type's name or doc comment.

**No-self-certification:** the agent or module that implements a check may not be the same
agent or module that approves its own implementation as satisfying an independent contract,
mutant suite, or structural audit. Approval must come from a role independent of the
implementer — see `hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`, and
`von-neumann-bypass.md` for the constitutional roles that carry this independence, and the
`mutant-kill-protocol` / `object-code-audit` skills for the evidence they must produce.

## Falsifier

Any of the following observed behaviors falsifies this rule on the object where it occurs:

- a function documented, named, or typed as a selector/observer/proposer that writes to
  persistent state or returns a freshly-minted authority-bearing value on any input path;
- an authority-bearing type with a public field, or with a safe construction expression
  reachable from outside its owning crate, that yields a value structurally equal to one from
  the admission function;
- a derive on an authority-bearing type that compiles to a second constructor (e.g.
  `#[derive(Default)]` yielding a valid-looking token, or `#[derive(Deserialize)]` accepting
  attacker-controlled bytes into the type without going through admission);
- more than one reachable production constructor path for a "certified adaptive mutation"
  authority type, found by exhaustive counting;
- a completion/standing claim ("ALIVE", "passes independent audit", "kills all mutants") where
  the approving party and the implementing party are the same agent, module owner, or session.

## Required Evidence

- A per-symbol construction-site count for each authority-bearing type (every `pub fn`,
  `pub(crate) fn`, derive impl, and trait impl capable of producing a value of that type),
  showing exactly one production path. Counting, not description, is the evidence.
- For the SELECT/DO boundary: a call-graph or compiled-artifact trace (per the
  `object-code-audit` skill) showing the selector/observer function has no write effect and no
  authority-minting effect on any reachable path.
- For no-self-certification: an audit or mutant-kill record (per `mutant-kill-protocol`,
  `evidence-report`) attributed to a role distinct from the implementer, with that distinction
  named explicitly in the record.

## Standing Consequence

A component that violates any clause above cannot be reported at ALIVE or BRANCHLESS_ALIVE
standing regardless of test pass rate. Any prior standing claim made without the counting and
independent-approval evidence above is REPORTED, not confirmed, and must be re-derived once the
evidence exists.

## Nonclaims

This rule does not specify which agent role performs which check, the enforcement mechanism or
its cost tier (see the hook/enforcement spec), or the current pass/fail status of any concrete
type or function in this repo (see the release ledger). It does not cover performance,
branchlessness, or side-channel properties of DO-side code, which are governed by sibling rules
and the `object-code-audit` / `mutant-kill-protocol` skills.
