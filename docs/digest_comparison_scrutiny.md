# Digest Comparison Scrutiny in BCINR (Rule 14)

## 1. Context: Rule 14 and the Deterministic Substrate
Under **Rule 14 (Numeric-law requirements)** of the `AGENTS.md` constitution, all authoritative operations must be fixed-width, deterministic, and bounded. Among the critical primitives singled out for "special scrutiny" is **digest comparison**. 

BCINR acts as the "hard substrate" for AGI, requiring that timing side-channels be physically impossible. The **Radon Law ($CC=1$)** mandates that the authoritative call graph must be entirely devoid of data-dependent branches (`if`, `match`, or early-exit `loop`s).

## 2. The Danger of Branching in Digest Comparisons
A standard string or byte array comparison (like `memcmp` or a naive `for` loop) uses a short-circuiting mechanism: it iterates through the bytes and immediately returns `false` upon finding the first mismatched byte.

### The Timing Side-Channel
This early-exit behavior causes the execution time to vary based on the input data. An attacker interacting with the system can measure how long the validation takes:
* A mismatch on the 1st byte returns instantly.
* A mismatch on the 16th byte takes measurably longer.

During cryptographic certificate validation (e.g., checking an `AcceptedCertificate` or `AcceptedEnvelopeReceipt` required by **Rule 11: ReceiptSound law**), an attacker can exploit this latency to iteratively guess a valid cryptographic digest byte-by-byte. This drastically reduces the attack surface, allowing an adversary to forge signatures or certificates without breaking the underlying cryptographic algorithm itself.

## 3. The Branchless Requirement ($CC=1$)
To prevent this, digest comparison must be transformed into straight-line arithmetic that executes in **strictly constant time**, regardless of whether the digests match entirely, partially, or not at all. 

### Constant-Time Execution Mechanics
A compliant implementation under the jurisdiction of `@von_neumann_bypass` utilizes bitwise operations instead of sequential control flow:
1. **Bitwise XOR**: Each corresponding byte of the two digests is XORed. Matching bytes yield `0`, while differing bytes yield non-zero.
2. **Unconditional Accumulation**: The XOR results of all bytes are accumulated (typically via bitwise OR) into a single integer without any short-circuiting.
3. **Mask Generation**: The final accumulated integer is arithmetically converted into a full-width execution mask (e.g., `0xFF...FF` for a match, `0x00...00` for a mismatch), strictly adhering to **Rule 9: Mask-based execution law**.

Because every byte is processed unconditionally, the CPU instruction trace, memory access pattern, and execution time remain identical for all possible inputs.

## 4. Enforcement and Security Guarantees
By eliminating data-dependent branches in digest comparison, BCINR closes the timing side-channel at the architectural level. This ensures that the system's autonomic loop (`Observe` $\rightarrow$ `Infer` $\rightarrow$ `Propose` $\rightarrow$ `Accept` $\rightarrow$ `Execute`) remains mathematically opaque to timing-based introspection.

This level of scrutiny is strictly enforced by the framework's constitutional gates:
* **`@turing_machine`**: Audits the generated machine code to verify $CC=1$ (zero conditional jumps or loop backedges).
* **`@armstrong_fault`**: Deploys hostile mutants (e.g., testing partial matches or stale digest acceptance) to verify that failures yield a lawful typed refusal like `DigestMismatch` or `CertificateStale`.
