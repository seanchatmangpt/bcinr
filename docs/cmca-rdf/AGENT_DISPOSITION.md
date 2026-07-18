# Agent Disposition — Four Constitutional Agents vs. v26.7.17 CMCA Release-Gate Topology

Scope: classifies `hoare-oracle`, `turing-machine`, `armstrong-fault`, `von-neumann-bypass`
against the five-agent mission topology (`cmca-numeric`, `cmca-authority`, `cmca-semantics`,
`cmca-verifier`, `cmca-release-integrator`), which is the sole ReleaseOwner set for gates
G0-G9 of the v26.7.17 CMCA release. This document is release-scoped commentary, not a rule
and not an agent definition — it does not replace or restate the four agents' own files; it
references them by path and records the ownership boundary between them and the mission
topology.

## hoare-oracle

**Purpose:** Axiomatic proof lead and specification owner (AGENTS.md §4, Checkpoint 1). Fixes
preconditions, postconditions, invariants, refusal conditions, and independent oracle semantics
before any authoritative implementation begins.

**Tool authority:** `Read, Grep, Glob, Write, Bash`.

**Owned files:** `CONTRACT.md`, `HOARE_TRIPLES.md`, `ORACLE_INDEPENDENCE.md` (per feature
touched).

**Overlap with release gates:** The file at
`/Users/sac/bcinr/.claude/agents/hoare-oracle.md` contains no mention of `G0`-`G9`, no
`ReleaseOwner` claim, and no reference to the v26.7.17 CMCA mission topology — it predates
that topology. Its stated authority is contract/proof artifacts, not gate ownership. The
overlap is indirect: contract correctness is almost certainly an input the release gates
depend on (e.g., a numeric-law or semantics gate cannot close without a satisfied contract),
so its output feeds `cmca-numeric` / `cmca-semantics` without itself being a gate owner. The
agent's own "Exclusive authority" language is written at the artifact level (three named
`.md` files), not at the gate level, so there is no textual claim of ReleaseOwner(g) to
revoke — but the phrase "Exclusive authority" is broad enough that a reader could mistake it
for gate-owning authority absent explicit scoping.

**Disposition:** Narrow.

**Justification:** No direct overlap was found — the file never asserts ownership of a G0-G9
gate. It is Narrowed rather than Retained as-is because its "Exclusive authority" and "Required
output for every primitive" language is unqualified with respect to the new five-agent
topology; left unscoped, it could be read as authorizing hoare-oracle to close a release gate
on its own contract, which the mission law forbids (exactly one ReleaseOwner per gate, and
that owner must be one of the five named mission agents). Narrowing keeps hoare-oracle as a
consultative contract/proof authority the mission agents (chiefly `cmca-numeric` and
`cmca-semantics`) invoke, with no gate-closing power of its own.

## turing-machine

**Purpose:** Structural auditor and merge gatekeeper (AGENTS.md §4, Checkpoints 5 and 7). Runs
source/object-code audits, the cheat scanner, cyclomatic-complexity checks, and
gate-jurisdiction verification before any merge or standing claim.

**Tool authority:** `Read, Grep, Glob, Bash, Write`.

**Owned files:** `SOURCE_AUDIT.md`, `OBJECT_CODE_AUDIT.md`, `AUTHORITATIVE_CALL_GRAPH.md`,
`GATE_JURISDICTION.md`.

**Overlap with release gates:** This is the file with the most direct textual proximity to
gate language: it owns `GATE_JURISDICTION.md` and performs "gate-jurisdiction audit," and its
description says it acts "before any merge or standing claim" — i.e., at merge-gate boundaries
generally, not specifically G0-G9 of v26.7.17 CMCA. It does not name G0-G9, does not use the
term `ReleaseOwner`, and does not claim to be the CMCA release's owner of any gate. Its
"gate-jurisdiction audit" verifies that "the authoritative crate is inside every relevant
gate's jurisdiction" — a check that a gate's scope is correctly drawn, not a claim to own or
close that gate. This is the clearest candidate for confusion with the new topology because
the vocabulary ("gate," "jurisdiction") is shared, even though the referents differ (general
merge-gate structural checks vs. the five named CMCA release gates).

**Disposition:** Narrow.

**Justification:** Overlap is vocabulary-level, not authority-level: nothing in the file
claims turing-machine is ReleaseOwner(g) for any g in {G0..G9}, but the term "gate" recurs
five times across its exclusive-authority and required-verification sections without ever
being scoped away from the CMCA release-gate meaning. Under the mission law's "exactly one
ReleaseOwner per gate" constraint, an unscoped structural-audit agent that already produces a
`GATE_JURISDICTION.md` artifact is the single highest-risk case for silent dual-ownership.
Narrow it explicitly: turing-machine's audits (`CC=1`, object-code branchlessness, cheat-scan
jurisdiction) remain a review capability the mission agents (most likely
`cmca-verifier`) invoke and depend on, but turing-machine itself holds no ReleaseOwner(g)
seat and does not close a CMCA gate unilaterally.

## armstrong-fault

**Purpose:** Adversarial test architect and mutation owner (AGENTS.md §4, §19, Checkpoint
3/6). Designs hostile mutants, negative-domain fixtures, and refusal-path tests; runs the
hostile mutation protocol before any implementation is accepted.

**Tool authority:** `Read, Edit, Write, Bash, Grep, Glob`.

**Owned files:** the mutant ledger, `MUTANT_KILL_MATRIX.md`. Explicitly scoped to test files
and mutant fixtures — "you are not the implementation owner."

**Overlap with release gates:** No mention of G0-G9, `ReleaseOwner`, or the CMCA mission
topology. Its authority is bounded to mutant design/kill-verification ("Minimum mutant
requirement," "Typed-refusal requirement," "Hostile mutation protocol") and it explicitly
disclaims implementation ownership. A surviving mutant "sets project standing to
`MUTATION_GATE_FAILED` and blocks all feature work" — this is a standing-vocabulary term
(§28), not a claim that armstrong-fault owns a numbered release gate; it reports a blocking
condition rather than adjudicating gate closure itself.

**Disposition:** Narrow.

**Justification:** The overlap is narrower than turing-machine's but not zero: the ability to
set `MUTATION_GATE_FAILED` project-wide is a gate-relevant side effect even though the file
never names a specific G0-G9 gate or claims ReleaseOwner status. Because this status can block
"all feature work," it functionally intersects whichever CMCA gate (likely owned by
`cmca-verifier`) gates on mutation-kill evidence. Narrow rather than Retain-as-is: keep
armstrong-fault as the consultative mutation authority the mission agents invoke for kill
evidence, but the file should be read as feeding evidence into a mission agent's gate decision,
not as itself holding a ReleaseOwner(g) seat.

## von-neumann-bypass

**Purpose:** Authoritative implementation owner (AGENTS.md §4, Checkpoint 4). Writes
branchless, allocation-free, bounded-arithmetic and state-transition code once
hoare-oracle's contract, armstrong-fault's mutants, and the AGENTS.md §30 checkpoint order are
in place.

**Tool authority:** `Read, Edit, Write, Bash, Grep, Glob`.

**Owned files:** implementation source under `crates/*/src`.

**Overlap with release gates:** No mention of G0-G9, `ReleaseOwner`, or the CMCA mission
topology anywhere in the file. Its scope is purely the writing of implementation code under
constraints (§3, §9, §10, §18) and it explicitly disclaims self-certification: "You do not
approve your own code against @hoare_oracle's contract, @turing_machine's structural/object-code
audit, or @armstrong_fault's mutant kills." It never claims a gate-closing or standing-setting
role at all — standing is reported by the other three agents about von-neumann-bypass's code,
not by von-neumann-bypass itself.

**Disposition:** Narrow.

**Justification:** This file has the least overlap of the four — it owns only source code, not
any `.md` gate or standing artifact, and it already textually forbids self-certification of
its own gate-relevant claims. It is still Narrowed rather than Retained-as-is purely for
topology completeness and consistency with the other three: von-neumann-bypass remains the
consultative/implementation-craft role the mission agents (chiefly whichever agent implements
under `cmca-numeric`/`cmca-semantics` direction) invoke, with an explicit statement that it
holds no ReleaseOwner(g) seat for any of G0-G9 — even though, unlike the other three, no
language in the file itself needed correcting to reach that conclusion.

## Final Disposition Table

| Agent | Disposition | Rationale |
|---|---|---|
| hoare-oracle | Narrow | Owns contract/proof artifacts (`CONTRACT.md`, `HOARE_TRIPLES.md`, `ORACLE_INDEPENDENCE.md`); no G0-G9 or ReleaseOwner language, but unqualified "Exclusive authority" wording needs explicit non-gate-owning scope so mission agents (`cmca-numeric`, `cmca-semantics`) can invoke it without ambiguity. |
| turing-machine | Narrow | Owns `GATE_JURISDICTION.md` and performs "gate-jurisdiction audit" — the closest vocabulary overlap with the CMCA gate topology of the four, though it names no G0-G9 gate or ReleaseOwner claim; must be explicitly stripped of any inferred gate-closing authority and retained as `cmca-verifier`'s structural/object-code review capability. |
| armstrong-fault | Narrow | Can set project-wide `MUTATION_GATE_FAILED` standing that blocks feature work, a gate-relevant side effect, though it names no specific release gate or ReleaseOwner claim; retained as the mission agents' consultative mutation-kill authority, not itself a ReleaseOwner. |
| von-neumann-bypass | Narrow | Least overlap of the four (owns only `crates/*/src`, explicitly disclaims self-certification of gate-relevant standing); narrowed for topology consistency and to state explicitly it holds no ReleaseOwner(g) seat, retained as the mission agents' implementation-craft capability. |
