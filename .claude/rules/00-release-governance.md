# Release Governance — Terminal Gate and Ownership Law

## The Law

The terminal release gate for a CMCA release is one thing: a `cargo publish --dry-run`
that exits 0 against the final integrated repository coordinate. Green tests, a clean
package build, and a clean source/cheat scan are necessary preconditions — none of them,
singly or together, is terminal completion. Completion is the dry-run exit code, nothing
short of it.

Single-ownership law: for every release gate `g` in the gate sequence G0..G9, exactly one
mission agent is `ReleaseOwner(g)`. Any number of reviewers may examine a gate; at most one
agent may hold final ownership of it. Ownership does not transfer by silence — it transfers
only by an explicit handoff recorded in the release ledger.

Only the `cmca-release-integrator` agent may emit the terminal standing declaration for the
release, and only after every gate G0..G9 is green with evidence that has been reproduced by
an independent verifier — not self-reported by the agent that did the work.

## The Falsifier

This rule is violated by any observable instance of:

- A standing declaration of release completion emitted by an agent other than
  `cmca-release-integrator`.
- A terminal declaration made while any gate G0..G9 lacks a reproduced-evidence record, or
  rests only on the implementing agent's own report of its own output.
- Two or more agents each holding final ownership of the same gate at the same time (as
  opposed to one owner plus any number of reviewers).
- A completion or readiness claim in chat, commit message, or ledger entry that uses a
  forbidden phrase — `implemented`, `tests pass locally`, `should pass dry run`, `ready to
  publish`, `production ready`, or equivalent — without both a specific bounded claim
  (what exactly, over what scope) and a link to the reproduced evidence backing it. This
  follows the repo-global no-overclaiming conventions
  (`~/.claude/rules/no-overclaiming-conversational.md`).
- Any claim of release completion that is not, itself, an exit-0 `cargo publish --dry-run`
  transcript against the final integrated coordinate.

## Required Evidence Class

- For the terminal gate: a captured, reproducible transcript of `cargo publish --dry-run`
  showing exit code 0, run against the exact commit/coordinate being declared released.
- For each gate G0..G9: a reproduced-evidence record attributable to a verifier distinct
  from the gate's implementing agent — not the implementing agent's own test output pasted
  into the ledger.
- For any ownership claim: a single named `ReleaseOwner` per gate, recorded in the release
  ledger, with reviewers (if any) named separately and without final-ownership authority.

## Standing Consequence of Violation

A release declared complete without a reproduced exit-0 dry-run transcript has no standing
declaration — it reverts to REPORTED status and blocks the terminal gate until the transcript
exists. A gate with more than one final owner, or a completion claim using a forbidden
phrase without bounded scope and linked evidence, is itself a gate failure and must be
recorded as such in the release ledger, not silently corrected in place.

## Nonclaims

- This rule does not itself verify anything: it defines what verification must produce
  before completion may be declared, and holds no test results, file:line references, or
  current defect status. Those belong exclusively in the release ledger (mutable), never
  in this file.
- This rule does not define gate ordering, per-gate content, or the specific work each
  `ReleaseOwner` performs — see the sibling agent definitions under `.claude/agents/` and
  the ledger for current gate status.
- This rule does not adjudicate what counts as "branchless" or perform the underlying
  hostile-mutation, disassembly, or cheat-scan checks — see the `mutant-kill-protocol`,
  `object-code-audit`, and `cheat-scan` skills, and the constitutional agents
  (`hoare-oracle`, `turing-machine`, `armstrong-fault`, `von-neumann-bypass`) for those
  invariants.
