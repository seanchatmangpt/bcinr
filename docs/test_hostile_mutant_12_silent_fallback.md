I have inspected `hostile_mutants.rs` using `grep_search` and `view_file`. 

**Findings:**
`mutant_12` is **not currently implemented** in `hostile_mutants.rs` (or anywhere else in the `crates/` directory). However, based on the project's documentation found in `/Users/sac/bcinr/docs/test_hostile_mutant_12_silent_fallback.md`, here is the exact information detailing what `mutant_12` (M12) is intended to represent:

### M12: Silent Denominator Fallback

**Mathematical Law Broken**
This mutant intentionally violates the strict numeric laws governing uncertified approximations and boundary conditions (as defined in `AGENTS.md` Rule 14 and Rule 18):
- **No Unwitnessed Approximations:** It is strictly prohibited to silently inject an epsilon ($\epsilon$) to avoid a division-by-zero, or to gracefully fall back to a simpler heuristic when a denominator approaches zero.
- **Fixed-width Lawfulness:** "No unsupported input may... fall back to a simpler algorithm [or] return a plausible default." The logic must be deterministic and fully branchless without artificially smoothing denominators to avoid faults. 

**Expected Outcome / Refusal**
If a denominator is invalid or reaches an unsupported zero-domain, the authoritative runtime MUST produce an explicit **Bounded Typed Refusal** (e.g., `NumericRangeExceeded` or `UnsupportedDomain`). 

An implementation of `mutant_12` would intentionally insert this forbidden "silent fallback" (such as `denominator.max(epsilon)`). The dedicated oracle test for `mutant_12` would supply a fixture with an invalid denominator and assert that the outcome is strictly a typed refusal. If the mutant correctly bypasses this (evaluating successfully instead of refusing), the test identifies the breach of the mathematical contract and kills the mutant.
