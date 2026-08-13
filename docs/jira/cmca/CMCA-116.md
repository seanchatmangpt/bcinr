# CMCA-116: escort_distribution can silently return ~36-37% relative-error output with zero signal to the caller

**Type:** Bug / Design Gap
**Priority:** High

## Summary

CMCA-106 measured `power()`'s error bound across the full admitted domain
(`|q| <= 16`) but its own acceptance criteria offered two possible closures —
(a) document a hard numeric bound, or (b) add a refusal/flag path for
low-confidence results — and only (a) was done. `escort_distribution`'s only
domain check is a hard `|q| > MAX_LENS_MAGNITUDE` cutoff; it does not vary
with the now-measured error, so `q=15.9` (documented ~36% relative error) and
`q=0.1` (documented ~0.5% error) return the identical `Ok(...)` shape with no
way for a caller to distinguish a trustworthy answer from an untrustworthy
one.

## Context

Found by adversarial review of CMCA-106's implementation.

`crates/bcinr-cmca/src/escort.rs:71-90` (module doc) states the risk plainly
and explicitly declines to act on it: *"~36-37% relative error at `|q|` near
16 is large enough that a caller relying on `power`'s output there for
anything more precise than 'roughly which sibling dominates' should not trust
the magnitude — but changing `escort_distribution`'s `Ok`/`Err` behavior...
is a public API change with its own scope and was deliberately left undone
here."* `escort_distribution`'s domain check (`escort.rs:198`) is a flat
`|q| > MAX_LENS_MAGNITUDE` cutoff. CMCA-106's own acceptance criteria
(`docs/jira/cmca/CMCA-106.md`) named both closure options; the ticket was
marked Done having only exercised the "document a bound" option.

Given this crate is heading into production use (autofde-lab), a caller with
no visibility into per-call error magnitude cannot make an informed decision
about whether to trust a fractional-`q` escort result — "measured but
unmitigated" is a real, open gap, not a closed ticket.

## Acceptance Criteria

- [ ] Decide the actual closure: either (a) narrow `escort_distribution`'s
      admitted domain to a region with a tight, acceptable error bound
      (e.g. `|q| <= 4`, where error stays under ~8%) and refuse fractional
      `q` beyond it with a new typed refusal, or (b) add a
      confidence/error-estimate field to the returned distribution (or a
      parallel "was this the exact integer path or the approximate
      fractional path, and roughly how much error" signal) so a caller can
      decide for themselves.
- [ ] If (a): update `MAX_LENS_MAGNITUDE`'s role for `escort_distribution`
      specifically (may need to diverge from `cascade`'s `MAX_LENS_MAGNITUDE`
      if the two truly need different bounds — document why, per the
      existing MAX_LENS_MAGNITUDE domain-doc precedent from earlier this
      session).
- [ ] If (b): design the signal to not break existing callers
      (`escort_distribution`'s current `Result<Vec<NonNegativeFixed>,
      EscortRefusal>` signature) — likely an additive field or a new
      wrapper function.
- [ ] Add a test proving a caller can distinguish (via whichever mechanism
      is chosen) a high-error fractional-q call from a low-error one.

## Files likely touched

- `crates/bcinr-cmca/src/escort.rs`
- `crates/bcinr-cmca/src/allocator/mod.rs` (`power`, `escort_power` if a domain change is needed)

## Related

- CMCA-106 (the ticket whose acceptance criterion (b) this closes)
