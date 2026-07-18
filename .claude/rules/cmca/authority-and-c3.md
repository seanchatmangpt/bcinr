---
paths:
  - "crates/bcinr-cmca/src/observatory.rs"
  - "crates/bcinr-cmca/src/proposal.rs"
  - "crates/bcinr-cmca/src/shadow.rs"
  - "crates/bcinr-cmca/src/jump.rs"
  - "crates/bcinr-cmca/src/stability.rs"
  - "crates/bcinr-cmca/src/certification.rs"
  - "crates/bcinr-cmca/src/mode_switch.rs"
  - "crates/bcinr-cmca/src/allocator.rs"
---

# Authority and C3 — The Full Chain Never Collapses

Applies to every artifact implementing any hop of the proposal → admission → shadow-execution →
jump-analysis → stability-candidate → certificate-seal → dwell → certified-mode-switch chain,
whether that chain lives in one file or is spread across several. Generalizes the CMCA C3
authority model past any single module boundary: the invariants below bind the chain as a
whole, not any one file in isolation.

## Invariant 1 — Four authorities, four distinct implementations

Proposal, admission, certification, and actuation are four distinct authorities. No function or
type may perform more than one of these roles. In particular: a component whose contract is to
OBSERVE telemetry and PROPOSE a candidate transition (the eye of the system) must never itself
be the thing that MINTS a certificate, or the thing that admits a state as certified. Minting a
certificate and proposing the transition it certifies are, structurally, two independently
implemented steps connected by an explicit call or data-passing boundary — never the same
function body, the same `impl` block's private state, or a shared mutable field that lets one
role silently perform the other's action.

**Falsifier:** the observing/proposing component's own code path returns, constructs, or writes
a certificate-shaped or admitted-state-shaped value on any reachable input, without passing
through a separately implemented certification/admission function owned by a different type or
module boundary; or a single enum/struct is tagged as serving more than one of {proposal,
admission, certification, actuation} in its own contract.

**Required evidence:** a per-authority construction-site count (as in the cardinality method of
`30-authority-separation.md`) showing that certificate-shaped and admitted-state-shaped values
are producible only from the certification/admission function's own module, never from the
proposal or observation module; and a call-graph trace showing the proposing component has no
reachable path that constructs those value types directly.

**Standing consequence:** a chain where any two of {proposal, admission, certification,
actuation} share an implementation cannot be reported at ALIVE or BRANCHLESS_ALIVE standing.
Any prior claim that the authorities are separated is REPORTED, not confirmed, until the
construction-site count and call-graph trace above exist.

## Invariant 2 — Co-occurring telemetry conditions are a set, never a lossy enum

When a component's state can be simultaneously true along several independent telemetry
dimensions (e.g. multiple stability or jump conditions holding at once), the result type
returned to callers must preserve the full set of true conditions — a flag SET (bitset, struct
of independent booleans, or equivalent), not a single enum variant that can only name one
condition at a time. A single "primary" projection derived from that set is permitted only as
an explicit, separately named, separately tested priority order layered on top of the full set
— never as the sole representation.

**Falsifier:** an evaluation function whose return type can represent at most one condition,
observed (by test or by type inspection) to discard a second true condition when two or more
hold simultaneously on the same input; or a "priority" projection that exists in code but has no
accompanying test exercising a multi-true input and asserting which conditions were discarded
and why.

**Required evidence:** a property/table-driven test (or proptest oracle, per
`mutant-kill-protocol`) that constructs inputs with two or more independent conditions true
simultaneously and asserts the returned set contains all of them; a separate, named test for the
priority-projection function asserting its documented tie-break order against the same
multi-true inputs.

**Standing consequence:** a result type that collapses co-occurring conditions to one enum
variant, with no separately tested full-set representation underneath, is UNSUPPORTED for any
claim that downstream consumers can observe or act on more than the single reported condition.

## Invariant 3 — A seal must bind every enumerated domain-specific identity, not a subset

Sealing a certificate or authority token must verify EVERY one of the domain-specific bindings
the release ledger enumerates for this system — including, at minimum, graph identity,
generated-payload identity, kernel/numeric-profile identity, pricing/floor-law identity,
control-mode identity, comparison-derivation identity, and round identity, plus any further
binding the ledger adds over time. A seal that compares only a subset of these bindings, or that
compares two receipts to each other without tying either one back to the actual domain artifact
it claims to describe, is not a complete seal — it is a partial check wearing a seal's name. Any
single mismatched binding, among all enumerated bindings, must refuse sealing; there is no
"mostly matches" outcome.

**Falsifier:** a seal/certify function that returns a sealed value on an input where one or more
of the ledger-enumerated bindings is proven mismatched (by a mutant that flips exactly one
binding while holding the rest correct); or a seal function that compares receipt-to-receipt
fields without an independent recomputation from the underlying domain artifact for at least one
of the enumerated bindings.

**Required evidence:** a hostile-mutant kill matrix (per `mutant-kill-protocol`) with one mutant
per enumerated binding, each mutant flipping exactly that binding and holding all others fixed,
each showing the seal function refuses; and a Hoare-logic or equivalent proof (per
`hoare-oracle.md`) that the seal's precondition entails equality of every enumerated binding
against a domain-artifact-derived value, not merely against another receipt.

**Standing consequence:** a seal implementation that has not been shown, per binding, to refuse
on that binding's mismatch cannot be reported as a complete seal; it must be reported as
PARTIAL, naming exactly which bindings lack a kill-matrix entry.

## Invariant 4 — Dwell / temporal-gating is a proof, never a caller-supplied boolean

A sufficient-dwell-time-elapsed condition, or any other temporal-gating condition guarding a
transition, must be represented as a type or proof value that can only be produced by the code
that actually observes elapsed time (or an equivalent monotonic witness) for the specific
transition and round in question. It must never be representable as a bare `bool` (or
bool-equivalent, e.g. an untyped flag field) that a caller can construct or supply directly to
satisfy the gate.

**Falsifier:** any safe-Rust construction path — outside the timing-observation function's own
module — that yields a value the transition function accepts as proof of sufficient dwell,
without that path itself having consulted a monotonic clock or round counter for the specific
transition/round pair; equivalently, a transition function whose signature accepts `bool`,
`Option<()>`, or another caller-satisfiable type in the position where a dwell proof belongs.

**Required evidence:** a construction-site count (as in Invariant 1) for the dwell-proof type,
showing its only producer is the timing-observation function, parameterized by the specific
round/transition identity it attests to; a mutant test that attempts to forge a dwell proof for
a round it was not observed for, and is rejected.

**Standing consequence:** a transition gated by a bare boolean dwell flag is UNSUPPORTED for any
claim that the dwell requirement is actually enforced, regardless of whether the boolean happens
to be set correctly on the current call path.

## Invariant 5 — Atomic transitions leave every persistent byte untouched on rejection, independently tested

An atomic state transition (a certified-mode switch or equivalent) must, on rejection, leave
every persistent byte it could have touched completely unchanged from its pre-attempt value.
This all-or-nothing property must be independently tested — exercised by a test that attempts a
transition designed to fail partway through and then diffs the full persistent surface against
its pre-attempt snapshot — not merely asserted true because the implementation "looks" atomic
(e.g. because it uses a single assignment or a lock).

**Falsifier:** a test (or hostile mutant, per `mutant-kill-protocol`) that forces rejection at
any candidate failure point inside the transition and observes any persistent byte the
transition could have touched differ from its pre-attempt snapshot.

**Required evidence:** a byte-level or field-level pre/post snapshot diff test covering every
persistent location the transition's write set can reach, executed once per distinct rejection
cause the transition can produce (not just one rejection path); a mutant-kill record for at
least one mutant that injects a partial-write before the rejection point.

**Standing consequence:** an atomicity claim for a transition lacking this snapshot-diff test,
for every distinct rejection cause, is REPORTED, not confirmed.

## Nonclaim — Absence of an actuation surface is a structural finding, not an assumption

If this codebase contains no actuation/broker surface at all, the obligation that no slow-rail
path reaches actuation is vacuously satisfied. That vacuous satisfaction must itself be recorded
as a structural finding — "not-applicable, evidenced by an import/call-graph search that found
no actuation surface" — and never silently assumed true without performing that search, and
never fabricated as a finding that was not actually run. An unresearched absence and a
researched absence are different evidence states even when both conclude "not applicable."

**Falsifier:** any report claiming the no-slow-rail-to-actuation obligation is satisfied (whether
vacuously or otherwise) that does not cite the specific import/call-graph search performed and
its scope.

**Required evidence:** the search command or method used (e.g. `object-code-audit` symbol trace,
or an explicit grep/call-graph enumeration) and its negative result, attached to the standing
claim.

**Standing consequence:** a vacuous-satisfaction claim made without a cited search is REPORTED
only, and must be treated as unverified until the search is performed and its scope recorded.

## General Nonclaims

This rule does not specify which concrete file currently implements which hop of the chain, the
current pass/fail status of any seal, dwell, or atomicity test, the enforcement mechanism or its
cost tier (see the hook/enforcement spec), or which agent role performs which check (see
`hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`, `von-neumann-bypass.md`, and the
`30-authority-separation.md` sibling rule for role and cardinality method). Current standing for
any concrete type or function in this repo belongs exclusively in the release ledger.

## See Also

- `30-authority-separation.md` — the SELECT-is-never-DO law and construction-site cardinality
  method this rule reuses for Invariants 1 and 4
- `00-release-governance.md`, `10-standing-and-evidence.md`, `20-repository-safety.md` — sibling
  path-scoped rules in this control plane
- `mutant-kill-protocol`, `object-code-audit`, `evidence-report` skills — the evidence mechanisms
  cited above
- `hoare-oracle.md`, `turing-machine.md`, `armstrong-fault.md`, `von-neumann-bypass.md` —
  constitutional agent roles that carry independent-approval authority
