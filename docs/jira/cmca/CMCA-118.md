# CMCA-118: generator.py's comment-stripping silently corrupts TTL string literals containing '#', with zero test coverage

**Type:** Bug
**Priority:** Medium

## Summary

`generator.py`'s `parse_ttl` strips everything after the first `#` on every
line to remove comments — but this is not Turtle-comment-aware, so it also
truncates any quoted string literal that legitimately contains a `#`
character, silently, with no exception raised. Confirmed by direct
reproduction, not theory. This directly contradicts the same function's
careful "raise ValueError, don't guess" handling of every other unsupported
Turtle construct. Separately, `generator.py` (the sole parser for this
crate's production numeric constants) has zero automated test coverage of
its own.

## Context

Found by adversarial review of the CMCA-105 generator fixes.

- `crates/bcinr-cmca/generator.py:25-27`:
  ```python
  for line in lines:
      clean_lines.append(line.split('#')[0].strip())
  ```
- Reproduced empirically (ran directly against synthetic TTL):
  ```
  ttl: cmca:Bar cmca:note "value # not a comment" .
  => parsed value: '"value'   (truncated, quote unterminated, no exception)

  ttl: cmca:Qux cmca:label "A#B" .
  => parsed value: '"A'       (same corruption)
  ```
  Both produce a malformed, truncated string with zero error — the
  `val_match` regex fails on the mangled fragment, falls through to
  `val = obj` (the raw truncated text), and `parse_ttl` returns silently.
- Contrast: the same function rejects (loudly, via `ValueError`) multiline
  literals, language tags, blank nodes, collections, nested lists, named
  graphs, and relative IRIs (`generator.py:29-45`) — the `#`-in-literal case
  is the one silent corruption path in an otherwise defensively-written
  parser.
- Confirmed not currently triggered by production data: this crate's actual
  `ontology/cmca-rdf.ttl`/`generalization.ttl` only use `#` in real comments
  today — but this is latent, not impossible, and will silently corrupt any
  future literal containing a URL fragment, an ID, or a note with a `#`.
- Confirmed no test coverage exists: `grep -rl "generator.py"` in
  `crates/bcinr-cmca/` returns only the generated `.rs` outputs and one doc
  comment reference — no `test_generator*.py`, no `pytest`, no Rust test
  shelling out to check generator output against known inputs.
- Scientific notation and internal-whitespace-in-literals were checked and
  found NOT to be bugs (parse correctly) — ruling those out as part of this
  review.

## Acceptance Criteria

- [x] Fix `parse_ttl`'s comment-stripping to be literal-aware (don't strip
      `#` occurring inside a quoted string) — a minimal fix likely just needs
      to track whether the scan position is inside an open `"..."` before
      treating `#` as a comment start. Done via a new `strip_ttl_comment`
      helper that scans each line, tracks `in_string` across `"` boundaries
      (honoring `\"` escapes), and only treats `#` as a comment start when
      not inside a literal.
- [x] Add a regression test (a `pytest` file, or a small Rust test invoking
      the generator as a subprocess) covering: a literal containing `#`, and
      at least the other "Unsupported Turtle construct" rejections already
      in the function, to lock in current correct behavior going forward.
      Done: `crates/bcinr-cmca/tests/generator_ttl_comment_stripping.rs`
      (Rust test shelling out to `generator.py` as a subprocess, matching
      this crate's existing test convention since no Python test files or
      pytest config exist). Covers: `#` inside a literal surviving intact,
      a genuine trailing/whole-line comment still being stripped, the
      multiline-literal and language-tag rejections still firing, and both
      real `ontology/*.ttl` files still generating successfully. Manually
      verified (outside the automated suite) that generated output for both
      real ontology files is byte-identical before/after the fix except for
      the expected `GENERATOR_SOURCE_DIGEST` line (which hashes
      `generator.py` itself, and legitimately changes since the file
      changed) — confirming no regression, since neither file currently
      contains `#` inside a literal.
- [ ] Given this is the sole parser for this crate's production numeric
      constants, consider whether `generator.py` deserves broader test
      coverage as its own follow-up (this ticket's minimum bar is the `#`
      bug + a regression test for it, not a full test suite). Left
      unchecked/out of scope for this ticket, as the AC text itself allows.

## Files likely touched

- `crates/bcinr-cmca/generator.py`
- A new test file (Python or Rust, whichever matches this crate's existing conventions best)
