---
name: mutant-kill-protocol
description: Use when adding or changing an authoritative bcinr primitive and you need to run the hostile mutation protocol from AGENTS.md §19 — inject mutants, verify typed-refusal or oracle-mismatch kills, and record the mutant ledger. Triggers on "mutation test", "kill matrix", "hostile mutant", or before claiming a primitive is ALIVE/BRANCHLESS_ALIVE.
---

# Mutant kill protocol

Implements AGENTS.md §19 (hostile mutation protocol) and the minimum mutant requirement in §18.
Delegate design/ownership to `@armstrong-fault`; this skill is the mechanical procedure.

## Steps

1. **Identify load-bearing laws.** For the target file, list at least three laws whose violation
   should be independently detectable (sign, mask, normalization, index, digest, clamp, refusal
   bypass, etc. — see AGENTS.md §18 for the canonical list).
2. **Produce one mutant per law.** Each mutant is a syntactically plausible, real code change
   (not `assert_ne!`-only theater — CHEAT-009 in §16). Write it as an actual alternate build of
   the function, injectable through the real build path (a `cfg`-gated variant, a copied crate,
   or a test-only feature flag — never a mock of the function under test).
3. **Inject through the real build path.** Do not test a hand-written stand-in; build and run the
   mutated source.
4. **Run the normal suite** (`cargo test -p <crate> -- --nocapture` or the project's `make test`)
   against the mutant.
5. **Verify detection.** The suite must fail via a *specific* typed refusal
   (`assert_eq!(result, Err(StabilityRefusal::...))`) or a named oracle-postcondition mismatch —
   never a bare inequality assertion.
6. **Record the ledger entry** for each mutant:

   ```text
   mutant id
   source file
   changed law
   exact mutation
   expected detection
   actual detection
   test name
   receipt digest
   standing
   ```

7. **Roll up standing.** Any surviving mutant (i.e. detection step 5 fails) sets project standing
   to `MUTATION_GATE_FAILED` for the affected feature and blocks all feature work per §19 — report
   this exactly, do not describe it as "mostly passing."

## Output

`MUTANT_KILL_MATRIX.md` for the feature, in the ledger format above, one row per mutant.

## Boundaries

- Do not derive expected results from the implementation under attack — derive them from the
  independent oracle (`@hoare-oracle`'s contract).
- Do not self-certify: the implementation owner (`@von-neumann-bypass`) may not be the one who
  judges mutant adequacy (§27).
