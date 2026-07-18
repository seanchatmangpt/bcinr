# Repository Safety

## Law

No destructive git operation — `git reset --hard`, `git clean -fdx`, force-push, or any
rewrite of history containing another agent's commits — may be performed without explicit
user authorization obtained in the current session.

Files located under any `src/generated/` directory, or under any directory whose file
header states `DO NOT EDIT` or `GENERATED`, may only be changed by re-running their
authoritative generator. Hand-editing such a file is prohibited regardless of how small or
urgent the change appears.

A failing test may never be deleted, skipped, or weakened (loosened assertion, widened
tolerance, reduced coverage) in order to make a suite pass. A known issue may never be
silently reclassified as future work solely because it is difficult to fix; deferral
requires the same disclosure as any other standing claim (see the release ledger, not this
rule).

Any lint-suppression attribute (`#[allow(...)]` or equivalent) added to silence a lint must
carry an adjacent comment naming the specific defect being deferred and the reason
suppression is used instead of a fix. A suppression with no such comment is itself a defect.

## Falsifier

Any of the following observed in the repository or its history falsifies this rule:

- A history rewrite, hard reset, or force-push touching a commit authored by an agent other
  than the one performing the operation, with no recorded user authorization for that
  specific operation.
- A diff that hand-edits a file under `src/generated/` or under a `DO NOT EDIT`/`GENERATED`
  header, rather than regenerating it.
- A diff that deletes, `#[ignore]`s, or weakens a previously-failing test without also
  fixing the underlying defect the test was checking.
- A commit message, comment, or ledger entry that reclassifies a known defect as "future
  work" without a corresponding disclosure of difficulty as the stated reason.
- A lint-suppression attribute with no adjacent comment identifying the deferred defect.

## Required Evidence

- For destructive git operations: an explicit, contemporaneous user statement authorizing
  that specific operation (not a standing or inferred approval).
- For generated files: a re-run of the authoritative generator producing the changed
  content, with the generator invocation identifiable (not a manual diff to the output).
- For test changes: the failure this rule would otherwise forbid removing must instead be
  resolved by a fix, with the fixed suite run showing the same test passing for its
  original reason, not passing because it was altered to accept the defect.
- For lint suppressions: the adjacent comment itself, naming the defect and the deferral
  reason, is the evidence — a suppression without one does not satisfy this rule regardless
  of any other justification given elsewhere.

## Standing Consequence

A violation of this rule voids any standing claim of the affected artifact (test suite,
generated file, or commit history) as trustworthy pending correction. Work built on top of
a violation inherits the same voided standing until the violation is reverted or repaired
and re-verified.

## Nonclaims

This rule governs repository hygiene only: git history integrity, generated-file
provenance, test-suite integrity, and lint-suppression disclosure. It does not state or
imply any CMCA-specific numeric law, authority law, or generator law — those are defined in
the path-scoped `cmca` rules and are out of scope here. Current defect status, file:line
references, and release-progress facts belong in the release ledger, not in this rule.
