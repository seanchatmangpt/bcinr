# Branchless Digest Comparison in BCINR

## The Problem: Timing Side-Channels
In standard cryptographic digest validation, algorithms like `memcmp` or iterative `for` loops use short-circuiting. They return `false` immediately upon encountering the first mismatched byte. This early-exit introduces a **timing side-channel** where validation time varies based on the input. An attacker can exploit this to iteratively guess valid cryptographic signatures byte-by-byte, drastically reducing the attack surface.

Under the BCINR constitution (Rule 14 and the Radon Law `CC=1`), the authoritative call graph must be entirely devoid of data-dependent conditional branches (`if`, `match`, or early-exit `loop`s).

## Branchless Evaluation Mechanics
To completely eliminate timing side-channels, BCINR enforces that digest comparison executes in **strictly constant time**, regardless of whether the digests match entirely, partially, or not at all. This falls under the jurisdiction of the `@von_neumann_bypass` construct, which transforms sequential control flow into constant-time arithmetic.

The process of evaluating a `DigestMismatch` securely without an early-returning `if` loop involves three steps:

1. **Bitwise XOR**: Each corresponding byte or word of the two digests is XORed together. For matching bytes, the XOR yields `0`. For differing bytes, it yields a non-zero value.
2. **Unconditional Accumulation**: The XOR results of all segments are accumulated (typically via bitwise OR or counting set bits using `count_ones()`, as implemented in `mask_xor_reduce_u64.rs`) into a single integer. Crucially, this accumulation processes every single byte unconditionally, without short-circuiting.
3. **Mask Generation**: The final accumulated integer is arithmetically converted into a full-width execution mask (e.g., `0xFF...FF` for a match, `0x00...00` for a mismatch), strictly adhering to **Rule 9: Mask-based execution law**.

## Application in the State Machine
When evaluating a mode switch (e.g., in `crates/bcinr-cmca/src/mode_switch.rs`), the comparison checks (like `certificate == expected_certificate`) are converted into a flat boolean mask (`cert_ok`). To maintain branchless determinism:

1. The next state (`candidate`) is computed fully and unconditionally.
2. The flat booleans are combined into a master `admitted` predicate. 
3. A branchless masked selection updates the state: `let next = if admitted { candidate } else { *persistent };`.
4. If a digest mismatch occurred, the persistent state replaces itself with its exact original value, and a typed refusal like `ModeSwitchRefusal::CertificateDigestMismatch` is returned.

Because every byte is processed unconditionally and the candidate is computed regardless of the match outcome, the CPU instruction trace, memory access pattern, and execution time remain identical for all possible inputs.
