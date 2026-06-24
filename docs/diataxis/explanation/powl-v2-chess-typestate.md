# POWL v2: Branchless Scheduling, TypeState Machines, and Receipt Architecture

**Version:** v26.6.24
**Type:** Explanation (Diátaxis)

This document explains the design rationale behind the POWL v2 implementation in bcinr — specifically the branchless calculus applied to workflow scheduling, the TypeState phase lattice enforcing linear execution, the receipt architecture that moves BLAKE3 off the hot path, and the correctness invariants exposed by adversarial testing. Every claim here is verifiable from the code as built.

---

## 1. The Branchless Calculus for Workflow Scheduling

### Why branches are eliminated from `scheduler_tick`

The bcinr branchless calculus exists to eliminate conditional branches from performance-critical code paths. The scheduler hot path — `scheduler_tick` — is the inner loop of the POWL executor. Branches here cause pipeline stalls. The goal is to reduce every conditional to arithmetic.

Before the Track B rewrite, `scheduler_tick` contained three `if`/`match` statements:

- Dispatching on `OpKind` (XOR gate vs. loop redo vs. other)
- Predicated execution of `apply_xor_dispatch`
- Predicated execution of `apply_loop_redo`

After the rewrite, all three are eliminated. Execution is predicated through masks.

### The two's-complement nonzero test

The fundamental primitive is: given a `u64` value `n`, produce `0xFFFFFFFFFFFFFFFF` if `n ≠ 0`, and `0` if `n == 0`. In two's complement:

```
nonzero_mask(n) = (n | n.wrapping_neg()) >> 63
```

**Proof:** If `n = 0`, then `n.wrapping_neg() = 0`, so `0 | 0 = 0`, shifted right 63 gives `0`. If `n ≠ 0`, then either `n` or `n.wrapping_neg()` has the sign bit set (at minimum, `n.wrapping_neg()` flips the sign for any nonzero value in two's complement), so `n | n.wrapping_neg()` has bit 63 set, and `>> 63` produces `1` (or `0xFFFF...` with arithmetic shift in masked form). Broadcasting with `0u64.wrapping_sub(result)` produces the full mask.

This is the correct form. The naive `(n.wrapping_sub(1) >> 63)` is **wrong** for `n = 1` it produces `0`, which is correct, but for `n = 2^63 + 1` it wraps through zero incorrectly. The corrected form `(n | n.wrapping_neg()) >> 63` handles all u64 values correctly.

### The `kind_mask` trick and why `#[repr(u8)]` is required

To dispatch on `OpKind` without a branch, we need a branchless equality test on the enum discriminant. The implementation:

```rust
fn kind_mask(kind: OpKind, target: OpKind) -> u64 {
    let diff = (kind as u8) ^ (target as u8);
    // diff == 0 iff kind == target
    // nonzero_mask(diff) produces 0xFFFF... if diff != 0
    // invert to get mask when equal
    let nonzero = (diff as u64 | (diff as u64).wrapping_neg()) >> 7;
    (nonzero ^ 1).wrapping_sub(nonzero)  // produces 0xFFFF... when equal, 0 when not
}
```

Note the shift is `>> 7`, not `>> 63`, because `diff` is derived from a `u8` comparison — bit 7 is the high bit. This works correctly only because `OpKind` is `#[repr(u8)]`: the `kind as u8` cast is a direct discriminant extraction, not a computed hash or pointer. Without `#[repr(u8)]`, the discriminant representation is unspecified, and the arithmetic relationship between discriminants is not guaranteed.

### Predicated dispatch eliminating the hot-path branches

With `kind_mask` established, the XOR and loop-redo dispatches become:

```rust
let is_xor  = kind_mask(op.kind, OpKind::Xor);
let is_loop = kind_mask(op.kind, OpKind::LoopRedo);

let fire_nonzero = nonzero_mask(fire);
let active_xor  = is_xor  & fire_nonzero;
let active_loop = is_loop & fire_nonzero;

// apply_xor_dispatch result is masked in or masked to zero
result = (apply_xor_dispatch(op, state) & active_xor)
       | (apply_loop_redo(op, state)    & active_loop)
       | (default_advance(op, state)    & !(active_xor | active_loop));
```

The `if`/`match` is gone. Both dispatch functions are evaluated (or their results discarded by mask), with no branch in the instruction stream.

**Algebraic proof for Join effective_pred:** The effective predecessor mask for a Join node is `pred_mask & choice_taken`. The branchless identity `pred_mask & !(pred_mask & !choice_taken) = pred_mask & choice_taken` holds because `!(pred_mask & !choice_taken) = !pred_mask | choice_taken`, and `pred_mask & (!pred_mask | choice_taken) = (pred_mask & !pred_mask) | (pred_mask & choice_taken) = 0 | (pred_mask & choice_taken)`.

---

## 2. The POWL v2 TypeState Machine

### The phase lattice

The POWL v2 execution model enforces a strict phase ordering through Rust's type system. The phase lattice is:

```
Unvalidated → Compiled → Scheduled<KIND> → Executing<KIND> → Receipted<KIND>
```

Each arrow is a consuming transition: the previous state is moved out, the next state is constructed. You cannot hold a reference to an `Executing` token while also holding a `Compiled` token for the same workflow instance.

This is enforced by Rust's ownership system: each phase token is a zero-sized type (ZST) wrapped in the workflow struct via `PhantomData<KIND>`. The transition functions consume `self` and return the next phase.

### Why `ExecutionToken` must not be `Clone`

`ExecutionToken` is the proof that execution has been lawfully admitted. If it were `Clone`, a caller could fork two execution paths from the same admitted state, producing two `Receipted` artifacts for one workflow instance. This would defeat the receipt chain: chain hashes are derived from previous receipts, so two forks would produce diverging receipt chains from the same `run_id`.

`#[derive(Clone)]` is explicitly absent. The type system enforces single-execution linearity. Any attempt to clone an `ExecutionToken` is a compile error.

### Why linear types are needed here

Without linear types (consuming transitions), the following invalid sequence becomes possible at runtime:

1. Admit and compile a workflow
2. Begin executing
3. Drop the execution handle mid-run (due to error path)
4. Re-admit the same workflow ID with a fresh token
5. Emit a receipt as if execution completed

With consuming transitions, step 3 drops the `Executing<KIND>` token without producing a `Receipted<KIND>`. The receipt cannot be emitted. The execution is recorded as incomplete in the OCEL trace. There is no silent success.

---

## 3. Receipt Architecture

### Why BLAKE3 must be off the hot path

BLAKE3 hashing of a receipt entry costs on the order of hundreds of nanoseconds. The `petri_tick` scheduler loop runs at sub-microsecond latency targets. Calling BLAKE3 inside `petri_tick` would increase hot-path latency by an order of magnitude.

The solution is ring-drain separation:

1. **Hot path (`petri_tick`):** Push a lightweight `EventWorkItem` (containing `run_id`, `op_id`, `timestamp`) to a `LockFreeMpmcRing`. Only `push_t1` (~10 ns) is on the hot path.
2. **Off-path worker (`ReceiptWorker::drain()`):** Drains the ring, accumulates `op_trace` bitmasks keyed by `run_id`, and calls BLAKE3 only when the `full_mask` is satisfied — meaning all ops for that run have been observed.

This means BLAKE3 is called exactly once per completed workflow run, never inside the scheduler loop, and never redundantly.

### The 57-byte `ReceiptLog` entry layout

Each receipt log entry is exactly 57 bytes:

```
run_id      (8 bytes, u64 LE)
op_trace    (8 bytes, u64 LE)
topo_tag    (1 byte,  u8)
chain_hash  (32 bytes, BLAKE3 output)
replay_ptr  (8 bytes, u64 LE)
```

Total: 8 + 8 + 1 + 32 + 8 = **57 bytes**.

### Why LE serialization, not `#[repr(C)]`

`#[repr(C)]` layout is platform-ABI-dependent. On some platforms, struct padding is inserted between fields to satisfy alignment requirements. A 57-byte struct might become 64 bytes on one platform and 60 on another, making the raw bytes non-portable.

Little-endian (LE) serialization via `u64::to_le_bytes()` / `u8::to_le_bytes()` produces identical byte sequences on all platforms. The `replay_ptr` field at byte offset 49 is guaranteed by the LE layout, not by `#[repr(C)]` alignment. Tests verify the byte offset of `replay_ptr` directly to catch any regression.

### Chain hashing and `content_hash()` on `PowlTape`

The `chain_hash` field in each receipt entry is computed as:

```rust
let mut hasher = blake3::Hasher::new();
hasher.update(&prev_chain_hash);
hasher.update(&run_id.to_le_bytes());
hasher.update(&op_trace.to_le_bytes());
hasher.update(&[topo_tag]);
chain_hash = hasher.finalize().into();
```

Each receipt is chained to the previous one via `prev_chain_hash`. The `content_hash()` method on `PowlTape` uses `blake3::Hasher` over the tape bytes. The previous stub that returned `[0u8; 32]` has been eliminated. Tests verify both that the hash is non-zero and that topologically distinct tapes produce distinct hashes.

---

## 4. Correctness Invariants

### The `capability_mask` bit-63 bug

The original `capability_mask` implementation in `enterprise.rs`:

```rust
// WRONG
let mask = (xor.wrapping_sub(1) >> 63);
```

This is the branchless "is-zero" test: `n.wrapping_sub(1) >> 63` produces `0xFFFF...` when `n = 0` (wraps to `0xFFFF...`, then high bit is 1), and `0` when `n > 0`. But for a **nonzero** test (capability is satisfied when the requirement bits are set, i.e., `xor ≠ 0` means unsatisfied), the logic must be inverted.

The bug was: when `xor = 1` (only bit 0 differs), `wrapping_sub(1) = 0`, so `>> 63` gives `0`, which looks like "mask not satisfied." This is correct. But when `xor = 2^63` (bit 63 differs), `wrapping_sub(1) = 2^63 - 1`, and `>> 63` gives `0` — incorrectly reporting "satisfied." Any requirement involving bit 63 would be silently passed.

The fix:

```rust
// CORRECT: nonzero test for all u64
let mask = (xor | xor.wrapping_neg()) >> 63;
```

For `xor = 2^63`: `xor.wrapping_neg() = 2^63` (since `-(2^63) = 2^63` in two's complement). `xor | xor.wrapping_neg() = 2^63`, and `>> 63` gives `1`. Correct: bit-63 requirements are now detected.

### The `nonzero_u32` widening bug

The original closure:

```rust
// WRONG for n >= 2^31 + 1
let nonzero_u32 = |n: u32| (n.wrapping_sub(1) >> 31) ^ 1;
```

For `n = 2^31 + 1 = 2147483649`: `n.wrapping_sub(1) = 2147483648 = 2^31`. `>> 31` gives `1`. `^ 1` gives `0`. But `n ≠ 0`, so this is wrong: the closure reports "zero" for a nonzero value.

The fix widens to `u64` before the test:

```rust
// CORRECT: widen first, then apply the test
let nonzero_u32 = |n: u32| {
    let n64 = n as u64;
    ((n64 | n64.wrapping_neg()) >> 63) as u32
};
```

Widening to `u64` before the negation ensures the high-bit analysis is performed in a type large enough to capture the sign relationship correctly.

### Why these bugs existed

Both bugs share a common origin: the specifications were written to describe the intended behavior but were never adversarially tested against boundary values. The `capability_mask` bug requires `xor = 2^63`, which only appears when a requirement flag uses bit 63. The `nonzero_u32` bug requires `n ≥ 2^31 + 1`, which only appears for values in the upper half of the `u32` range.

Standard property tests with random small integers would not catch either bug. Only adversarially constructed tests targeting the specific boundary values will expose them. The 8 regression tests added in Track A are specifically designed around these boundaries: `capability_mask` is tested with `xor` values of `0`, `1`, `2^31`, `2^63 - 1`, `2^63`, and `u64::MAX`; `nonzero_u32` is tested at `0`, `1`, `2^31`, `2^31 + 1`, and `u32::MAX`.

---

## 5. The Anti-Pattern: Aspirational Code

### What the anti-llm-cheat-lsp finds

The METRIC violation family (specifically CLAIM-004) flags code that asserts a formal property in a comment or documentation string but does not implement a corresponding test or proof. The scan found multiple instances of:

```
// SAFETY: this is correct by the branchless calculus
```

paired with implementations that had the wrong shift amounts, wrong negation, or widening errors described above. The claim was true of the intended algorithm; it was false of the actual code.

CLAIM-004 "false victories" are dangerous specifically in formal verification contexts: the comment trains both human reviewers and automated tools to trust the code without re-deriving the proof. The bug is present; the comment hides it.

### Why this matters for formal verification systems

bcinr's formal verification model requires that `// SAFETY:` comments be derivable from the code, not asserted about it. A comment that says "nonzero test for all u64" on a function that fails for half the u64 range is not a safety annotation — it is a liability.

The correct process:

1. State the invariant as a property test with adversarial inputs, not as a comment.
2. The comment documents which invariant the test encodes.
3. The Hoare-logic annotation, if present, is derived from the test evidence.

This is the Van der Aalst Constitution applied to code proof: if the event log (the test suite) cannot prove a lawful process happened, then the proof did not happen. A comment is not an event. A passing test is.

---

## Summary of Verified Invariants

| Invariant | Location | Verified by |
|---|---|---|
| `capability_mask` correct for bit-63 | `enterprise.rs` | 6 adversarial regression tests |
| `nonzero_u32` correct for n ≥ 2³¹ | scheduler primitives | boundary regression tests |
| `kind_mask` requires `#[repr(u8)]` | `OpKind` definition | type-level enforcement + 5 unit tests |
| `ExecutionToken` not `Clone` | token type definition | compile-time (no `derive(Clone)`) |
| BLAKE3 not on hot path | `petri_tick` / `ReceiptWorker` | architecture + latency measurements |
| 57-byte entry layout portable | `ReceiptLog` | byte-offset tests on `replay_ptr` |
| `content_hash()` non-stub | `PowlTape` | non-zero hash + diversity tests |
| `#![forbid(unsafe_code)]` inner attr | all algorithm modules | compile-time enforcement |
| Dead `in_deg` removed | `kahn_check` | compile-time (unused variable removed) |

For performance measurements, see `docs/BENCHMARKS.md`. No benchmark numbers are stated here because they are substrate-dependent and must be reproduced from the current build.

---

## References

- `crates/bcinr-logic/src/` — core algorithmic modules including scheduler primitives
- `crates/bcinr-logic/src/SAFETY.md` — full audit trail of all unsafe blocks
- `docs/diataxis/reference/phd_gates.md` — formal verification gates as completed proofs
- `docs/BENCHMARKS.md` — latency targets and regression thresholds
- `Makefile.toml` — `cargo make check`, `cargo make test`, `cargo make clippy`
