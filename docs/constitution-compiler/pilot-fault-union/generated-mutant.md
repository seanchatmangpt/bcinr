# Generated mutant entry — pilot illustration

**Status:** ILLUSTRATIVE. Generated from ConstitutionIR claim `cmca.numeric.fault-join-semilattice`
(see `claim.yaml` in this directory), following the row/format conventions visible in the real,
currently-being-updated `/Users/sac/bcinr/crates/bcinr-cmca/MUTANT_KILL_MATRIX.md` (read
read-only for format reference; this pilot file is NOT an entry in that real matrix, is not
appended to it, and describes a mutant that has not been authored or run against the real crate).

## Mutant identity

`fault-union-restore-first-error-sentinel`

(Named identically to the constitutional-mutant surface already listed, but marked
`INFRASTRUCTURE_BLOCKED`, in the real matrix's "New constitutional-mutant surfaces requested"
table — this pilot file is what that row's full description *would* look like if the real
crate's compile break were resolved and the mutant could actually be authored and run.)

## Row (real-matrix format)

| mutant id | source file | changed law | exact mutation | expected detection | actual detection | classification | standing |
|---|---|---|---|---|---|---|---|
| `fault-union-restore-first-error-sentinel` | `crates/bcinr-cmca/src/allocator.rs` (composition site) | `cmca.numeric.fault-join-semilattice` | Replace the fault-set union at the sequential-composition site with "keep the first non-empty fault set seen, discard the second" (a first-error-wins sentinel accumulation) | `generated-property-test.rs`'s `disjoint_single_faults_both_survive_composition` fails: composed set contains only `tag_a`, missing `tag_b` | *(not run — this pilot's property test does not target the real crate; see Nonclaims below)* | *(not run)* | *(not run — ILLUSTRATIVE ONLY)* |

## Mutation description

Change the sequential-composition operator's fault-handling from:

```
faults(a ; b) = faults(a) ∪ faults(b)
```

to:

```
faults(a ; b) = faults(a) if faults(a) is non-empty, else faults(b)
```

i.e. restore a "first error wins" accumulation discipline: once one step in a composed chain has
raised any fault, no fault from any subsequent step is recorded, even if that subsequent fault is
a distinct, non-overlapping condition.

## Expected detection condition

Per claim.yaml's `falsifier_family[0]`: construct step `a` raising fault tag `F1` only and step
`b` raising a distinct, non-overlapping fault tag `F2` only. Compose `a ; b`. Under the mutation,
the composed result's fault set is `{F1}` — a strict subset of the correct `{F1, F2}` union,
with `F2` silently dropped. The property test `disjoint_single_faults_both_survive_composition`
in `generated-property-test.rs` is constructed to assert `composed.contains(&tag_b)` precisely to
catch this; under the mutation that assertion fails, killing the mutant.

Symmetric mutation not separately enumerated here but implied by the same claim: "keep only the
*last* fault seen" (dropping `F1` instead of `F2`) is an equally valid mutant of the same
invariant and would be caught by the same property test's `composed.contains(&tag_a)` assertion.
A real generator would likely need to emit both as distinct mutant ids
(`...-first-error-sentinel` and `...-last-error-sentinel`), since claim.yaml's single falsifier
family covers both directions in prose ("must never collapse to... first... or... last") but the
matrix format wants one row per concrete mutation, not one row per invariant. This one-to-many
expansion (one falsifier family -> two distinct mutant rows) is a friction point noted in
SUMMARY.md.

## Nonclaims

This mutant has not been authored as a `#[cfg(feature = ...)]` mutant against the real
`crates/bcinr-cmca/src/allocator.rs`, and has not been run. No production source under
`crates/bcinr-cmca/src/` was touched to produce this file. Per the real matrix's own current
finding, the real crate does not compile in its present working-tree state (259 `from_bits`/`val`
API-break errors), so even if this mutant were authored today against the real tree, it could not
be exercised to completion — the real matrix's own `INFRASTRUCTURE_BLOCKED` classification for
its structurally identical `fault-union-restore-first-error-sentinel` row applies here too, and
this pilot file does not change that fact or attempt to fix it.
