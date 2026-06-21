# Side-Channel Resilience and Why Timing Matters

A function can be *functionally* correct and still *leak*. If how long it
runs, or which memory it touches, depends on a secret, an observer who can
measure that running time or cache state can recover the secret without ever
seeing it directly. This document explains why branchless code is the
natural defence, and — just as important — where branchless code alone is
*not* enough.

## Timing is an output you did not declare

Think of a function as having two outputs: the value it returns, and the
*time* it takes. The second is implicit and usually ignored, but it is
observable. The canonical disaster is the early-exit comparison:

```rust
// LEAKY: returns as soon as a byte differs.
fn eq(a: &[u8], b: &[u8]) -> bool {
    for i in 0..a.len() {
        if a[i] != b[i] { return false; }   // time reveals the match prefix
    }
    true
}
```

An attacker submitting guesses against a secret learns, from the *latency*,
how many leading bytes matched — and can recover the secret byte by byte.
The bug is not in the boolean result, which is perfect; it is in the timing,
which encodes the secret.

## Two channels: control flow and memory

Secret-dependent leakage enters through two doors.

- **Control flow.** A branch whose direction depends on a secret produces
  secret-dependent timing — directly (different path lengths) and indirectly
  (the branch predictor's *state* now depends on the secret, perturbing
  later timing). This is the door `theory-1.md`'s misprediction story opens.
- **Memory access.** A load whose *address* depends on a secret produces a
  secret-dependent cache footprint; an attacker probing cache state learns
  the address, hence the secret. Data-dependent table indexing is the usual
  culprit.

A side-channel-resistant primitive must close *both* doors: no
secret-dependent branch, **and** no secret-dependent memory address.

## How branchless closes the first door

Encoding the conditional as a mask (`theory-3.md`) removes the
secret-dependent branch. The constant-time comparison becomes:

```rust
let mut diff = 0u32;
for i in 0..len {
    diff |= (a[i] ^ b[i]) as u32;   // accumulate; never short-circuit
}
is_zero_mask_u32(diff)              // all-ones iff every byte matched
```

Every byte is always examined; the loop length depends on the buffer size,
not on the data; the result is derived by a branchless mask. The timing now
reveals only the length, which was never secret. This is the same shape as
`bitset.rs`'s `hamming_u64_slices` and `parse.rs`'s fixed-width
`parse_hex_u32`, both of which scan a fixed extent rather than stopping at
the first interesting byte.

## The second door is the subtler one

Branchlessness is *necessary* but not *sufficient*. A branchless function
that indexes a table with a secret still leaks through the cache. bcinr's
table-driven code is written to keep the *access pattern* data-independent
where the index could be sensitive: `dfa_advance` masks an out-of-range
index down to a fixed slot rather than branching to an error path, but the
deeper guarantee — that the *sequence* of touched addresses does not encode a
secret — is a property the caller must preserve by how they use the table.
The honest statement is: removing the branch removes the control-flow
channel; removing the *data-dependent address* removes the memory channel,
and that is an additional obligation.

## Scope, and what this is not

Two caveats keep the claim truthful.

1. **The compiler can reintroduce a branch.** Source-level branchlessness is
   not a guarantee of machine-level branchlessness; an optimiser may lower a
   masked select back into a conditional, or a `cmov` into a branch. Genuine
   constant-time guarantees require inspecting the emitted assembly, which is
   why the library's claims are tied to specific lowerings (e.g. the `SETB +
   NEG` note in `mask.rs`) and verified, not assumed.
2. **Microarchitecture has other channels.** Power draw, port contention,
   and speculative execution (Spectre-class) are real and are *not* fully
   addressed by data-independent timing alone. bcinr's primitives remove the
   *algorithmic* timing and access-pattern channels; they are a foundation
   for side-channel resistance, not a blanket immunity certificate.

The takeaway: branchless code makes *timing* a non-output and is the
prerequisite for constant-time cryptographic-style code. Treat it as the
floor of a side-channel argument, not the ceiling — the WCET discussion in
`theory-7.md` is the same property viewed through a real-time, rather than a
security, lens.
