# Object-Code Audit — `bcinr-cmca` Authoritative Allocation Root

**Owner of this report:** cmca-verifier
**Scope:** `bcinr_cmca::allocator::allocate` (the authoritative allocation root, per
`crates/bcinr-cmca/src/allocator.rs` doc comment: "This is the entry point for the Cascade
Allocation engine") and its direct/transitive callees.
**Method:** dedicated linked-executable harness (per `object-code-audit` skill and this task's
instructions), because a prior attempt to disassemble the `.rlib` member directly found the
rustc rlib archive member format undecodable with otool-classic on this machine.

## Standing: BLOCKED — crate does not compile in this git state (pre-existing, not introduced by this audit)

This is reported honestly rather than fabricating a symbol table, per the task's explicit
instruction and per `no-overclaiming-rust.md`: "Unimplemented ≠ stub-that-returns-success;
... loud not quiet."

## What was built

A new binary target was created at `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/`:

- `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/Cargo.toml` — path-dependency on
  `bcinr-cmca` (`../../crates/bcinr-cmca`), added to the workspace `members` list in
  `/Users/sac/bcinr/Cargo.toml`.
- `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/src/main.rs` — `main()` calls
  `bcinr_cmca::allocator::allocate(...)` exactly once, with the same fixed sample inputs as
  the crate's own doctest fixture (`OBJECT_REGISTRY`, `LENS_REGISTRY`, `LAMBDA`, `ETA`, an
  all-root `parent` array, zeroed `mu`/`costs`, and an `AdaptiveUpdate` proof token), then
  folds the returned `AllocationOutcome` (candidate array, numeric-fault bits, refusal bits,
  plus the two output parameters `last_switch_t`/`prev_mode`) into a `u64` checksum that is
  printed via `println!`. The print is the anti-dead-code-elimination sink the task asked
  for; no other code touches the result.

This harness scaffold itself is source code only — it has not been exercised, because the
dependency it links against does not compile (see below). Its correctness as a harness is
therefore also **UNVERIFIED**, not just the audit it was meant to produce.

## Reproduction: `bcinr-cmca` fails to compile, in both debug and release, independent of this harness

Command run first (debug, to isolate the harness from the release-profile question):

```
$ cargo build -p bcinr-cmca
```

Exit: non-zero (`error: could not compile `bcinr-cmca` (lib) due to 259 previous errors; 6
warnings emitted`).

Command run second (release, the actual profile the audit needs):

```
$ cargo build --release -p bcinr-cmca
```

Exit: non-zero, same failure mode (`error: could not compile `bcinr-cmca` (lib) due to 259
previous errors; 6 warnings emitted`).

Command run third (the harness itself, release):

```
$ cargo build --release -p bcinr-cmca-audit-harness
```

Exit: non-zero. Fails for the identical reason — `bcinr-cmca` (the harness's only
dependency) fails to build as a lib crate before the harness binary itself is ever reached.
The harness's own `main.rs` produced zero compiler diagnostics of its own; all 259 errors are
inside `bcinr-cmca`'s existing source files.

### Error breakdown (from the debug build; release build produces the same set)

```
219  error[E0599]: no associated function or constant named `from_bits` found for struct `fixed::NonNegativeFixed`
 22  error[E0616]: field `val` of struct `fixed::NonNegativeFixed` is private
 13  error[E0599]: no associated function or constant named `from_bits` found for struct `SignedFixed`
  5  error[E0616]: field `val` of struct `SignedFixed` is private
```

Per-file distribution of the call sites (`file:line` of the diagnostic, not of the
definition):

```
crates/bcinr-cmca/src/observatory.rs — 25 call sites (lines 276,280,282,284,289,292,384,386,
                                        401,402,404,417,418,420,437,438,440,443,451, plus
                                        repeats)
crates/bcinr-cmca/src/lrc.rs         — 6 call sites (lines 45,46,47,48 — this is the module
                                        added in commit 3338f59a "Add LRC module, hostile
                                        mutant tests, and implementation reports for
                                        bcinr-cmca", the tip of the current branch)
```

### Root cause

`crates/bcinr-cmca/src/fixed.rs` defines `NonNegativeFixed`/`SignedFixed` with:

- `pub const fn from_value_bits(bits: u32) -> Self` (line 145) — **not** `from_bits`.
- `pub(crate) const fn from_parts(val: u32, faults: NumericFaultSet) -> Self` (line 133 is
  the struct definition; the `val` field itself is private, no `pub` accessor by that name
  exists for direct field read).

But `observatory.rs` and `lrc.rs` call `NonNegativeFixed::from_bits(...)` /
`SignedFixed::from_bits(...)` (a name that does not exist anywhere in `fixed.rs`) and read
`.val` directly on values of these types (a private field). This is not a subtle
off-by-one — it is a wholesale API-name mismatch between `fixed.rs`'s current public surface
and two of its callers, affecting **259** call sites. `rustc` itself suggests the fix
(`from_value_bits`) in every diagnostic, which confirms the mismatch is mechanical rather
than a deeper semantic disagreement — but confirming intent (is `.val`/`from_bits` the
callers' bug, or was `fixed.rs`'s public surface changed out from under them) is outside
`cmca-verifier`'s authority: `fixed.rs`'s numeric API surface is `cmca-numeric`'s domain
(see `AGENTS.md` §2 constitutional precedence and the no-self-certification constraint this
task was launched under). Silently editing `fixed.rs` or `observatory.rs`/`lrc.rs` to make
the crate compile would be exactly the kind of unreviewed numeric-hot-path change that
`numeric-hot-path.md` and the authority separation rules exist to prevent, and it is not
this task's assignment.

## Consequence for this audit

Because `bcinr-cmca` cannot be built as an rlib, it cannot be linked into any executable,
release or debug, on this branch as it stands. The chain the task specified —

```
build harness --release → otool -tv the linked executable → per-symbol table
```

is **un-runnable at its first step**, not at the disassembly step. No `.o`/executable exists
to disassemble. Any claim of a clean or dirty per-symbol table for `allocate` and its
callees at this time would be fabricated. None is given.

`uname -s` on this machine reports `Darwin`, so the disassembler that *would* have been used
once a release binary exists is `otool -tv` (not `objdump -d`), consistent with the task's
platform-detection instruction — this is recorded for the next attempt, not exercised here.

## Per-symbol table

**Not produced.** Standing: **UNKNOWN**, for the structural reason above (dependency crate
does not compile), not a decoding/inlining failure of the kind anticipated by the task
("inlining collapses everything into one symbol"). This is a compile-time blocker, one step
before the disassembly step could even be attempted.

## What would unblock this

1. `cmca-numeric` (or whichever role owns `fixed.rs`/`observatory.rs`/`lrc.rs`) reconciles
   the `from_bits`/`from_value_bits` and `.val`-private-field mismatches so
   `cargo build --release -p bcinr-cmca` exits 0.
2. Re-run `cargo build --release -p bcinr-cmca-audit-harness` (harness already exists at
   `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/`, already wired into the workspace).
3. `otool -tv target/release/bcinr-cmca-audit-harness` (Darwin) to get real disassembly of
   the linked executable, then build the per-symbol table (symbol name, conditional-jump
   count, loop-backedge count, panic-path presence, allocator-symbol presence, standing) as
   originally specified.

## Files touched by this task

- `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/Cargo.toml` (new)
- `/Users/sac/bcinr/tools/bcinr-cmca-audit-harness/src/main.rs` (new)
- `/Users/sac/bcinr/Cargo.toml` (added `tools/bcinr-cmca-audit-harness` to workspace
  `members`)
- `/Users/sac/bcinr/crates/bcinr-cmca/OBJECT_CODE_AUDIT.md` (this file, new)

No file inside `crates/bcinr-cmca/src/` was modified. No test was deleted or skipped. No
`#[allow(...)]` was added. Nothing was committed (`cmca-verifier` does not commit per this
task's constraints).
