---
paths: ["crates/bcinr-cmca/tests/**"]
---

# Verification Standards for bcinr-cmca Test Suites

## Invariant 1: A mutant kill requires a named-law assertion

A mutant (a deliberately corrupted implementation variant) is a valid kill-test only if its
detection assertion names the specific violated postcondition or typed refusal the corruption
is expected to trigger. An assertion that merely compares baseline output against mutant output
for inequality proves that the two outputs differ; it does not prove that the correct law is
what caught the difference, and it is not acceptable as the sole detection mechanism.

**Falsifier:** A test in this suite passes (reports "mutant killed") when the only assertion
present is a bare `assert_ne!`/inequality/diff check between baseline and mutant output, with no
assertion tied to a named postcondition, invariant, or typed-refusal variant.

**Required evidence:** Each kill-test's assertion must reference a specific, named postcondition
or a specific typed-refusal variant (e.g., an enum discriminant, an error kind, or a documented
invariant identifier) that the corruption is expected to violate, and the assertion must fail to
compile or fail at runtime in a way traceable to that named law — not merely to "something
changed."

**Standing consequence:** A kill-test that satisfies only the bare-inequality pattern does not
count toward mutant-kill coverage for any primitive's standing. The mutant is recorded as
UNVERIFIED, not KILLED, until a named-law assertion is added.

**Nonclaims:** This invariant does not require that every test avoid comparing baseline and
mutant output — such comparisons may be included as supporting evidence. It requires only that
a named-law assertion also be present and that it, not the bare comparison, be the assertion of
record.

## Invariant 2: Oracle independence is structural and mathematical, not textual

An independent oracle must be structurally and logically distinct from the production
implementation it checks. Translating the production code line-by-line into a different numeric
type, or importing and wrapping the production function, does not establish independence — both
are the same algorithm restated, and both will reproduce the same defect the production code
contains. A genuinely independent oracle uses a different mathematical form entirely: a
closed-form formula, an arbitrary-precision reference implementation, a symbolic or SMT model,
or exhaustive enumeration over a reduced domain.

**Falsifier:** An oracle module in this suite that imports, calls, or line-for-line mirrors the
control flow of the production function under test — differing only in numeric type, variable
names, or formatting — while being described as "independent."

**Required evidence:** For each oracle, a statement of which of the four independent forms
(closed-form, arbitrary-precision reference, symbolic/SMT, exhaustive enumeration) it uses, and
a demonstration that its implementation path shares no control-flow or algorithmic structure
with the production code.

**Standing consequence:** A test relying on a non-independent oracle cannot be cited as evidence
of correctness beyond "the two code paths agree" — it may not be used to support any stronger
claim (e.g., "verified against an independent oracle") in a report or ledger entry.

**Nonclaims:** This invariant does not forbid oracles from being implemented in Rust or from
sharing test infrastructure (harnesses, fixtures, generators) with the production code. It
forbids only sharing the algorithmic derivation being checked.

## Invariant 3: Verification evidence is pinned to an exact reproducible coordinate

Every piece of verification evidence — a test result, a benchmark number, a disassembly, a proof
run — must be pinned to an exact reproducible coordinate: commit hash, toolchain version, target
triple, feature flags, and build profile. Evidence detached from its coordinate is not
transferable to a later coordinate without re-verification; a passing result recorded against one
commit or toolchain says nothing about a different commit or toolchain until re-run.

**Falsifier:** Evidence presented in a report or ledger entry without an accompanying commit
hash, toolchain identifier, target, feature set, and profile — or evidence whose recorded
coordinate no longer matches the artifact being described.

**Required evidence:** A coordinate tuple (commit, toolchain, target, features, profile)
attached to every recorded result, sufficient for a third party to reproduce the exact build and
run that produced it.

**Standing consequence:** Evidence without a pinned coordinate may not be cited to support a
standing claim (ALIVE, BRANCHLESS_ALIVE, or any other) at any coordinate other than the one it
was actually produced at; it must be re-verified before reuse.

**Nonclaims:** This invariant does not require re-running evidence at every commit — only that
reuse across commits be preceded by re-verification, not assumed by continuity.

## Invariant 4: Compile-fail suites support only the quantifier they actually test

A compile-fail (trybuild-style) test suite establishes only that the specific programs it
attempts fail to compile as expected. Such a suite must not be cited to support a universal
claim of unforgeability or uncircumventability; the claim must be scoped honestly to "these N
specific unsafe-construction attempts fail to compile." A broader claim about the full
construction surface requires combining the compile-fail results with a public-API surface
review.

**Falsifier:** A report or ledger entry that cites a fixed-size compile-fail test list to
support an unqualified universal claim (e.g., "cannot be constructed unsafely," with no
quantifier or scope named).

**Required evidence:** The exact count and description of attempted programs in the compile-fail
suite, stated as the quantifier of the claim, plus a separate, explicit public-API surface
review when a broader claim is intended.

**Standing consequence:** A universal-unforgeability claim resting solely on a finite
compile-fail list, without an accompanying API surface review, is treated as an overclaim and is
not eligible to support ALIVE or equivalent standing.

**Nonclaims:** This invariant does not diminish the value of compile-fail tests as evidence for
their own scoped claim — it forbids only extrapolating past that scope without the additional
review.

## See Also

- Mutant-ledger format (id, source file, changed law, exact mutation, expected detection, actual
  detection, covering test name, standing) is a release-ledger requirement, not restated here.
- `/Users/sac/bcinr/.claude/skills/mutant-kill-protocol/SKILL.md`
- `/Users/sac/bcinr/.claude/skills/object-code-audit/SKILL.md`
- `/Users/sac/bcinr/.claude/agents/hoare-oracle.md`
- `/Users/sac/bcinr/.claude/agents/turing-machine.md`
- `/Users/sac/bcinr/.claude/agents/armstrong-fault.md`
- `/Users/sac/bcinr/.claude/agents/von-neumann-bypass.md`
- `/Users/sac/bcinr/AGENTS.md`
