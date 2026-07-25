# Deterministic Entropy & Nonce Generation in BCINR

The BCINR architecture imposes strict absolute runtime laws: zero heap allocations, strict branchless execution ($CC=1$), and completely deterministic output for reproducible, mathematically certifiable workflows. To conform to these laws, authoritative hot paths are categorically barred from using non-deterministic operating system calls (e.g., `getrandom` syscalls, `/dev/urandom`, or time-based entropy).

Here is how BCINR securely handles pseudo-randomness and nonces without violating its axiomatic constraints:

## 1. External Entropy Injection via the "Slow Rail"

The hot path cannot generate true entropy. Instead, nonces and initial seeds must be generated externally (e.g., via the "slow rail", the client, or non-authoritative boundaries) and then injected into the deterministic environment as fixed inputs.

- **Nonces as State Inputs:** Globally unique nonces are embedded directly into state structures prior to hot-path admission. For instance, `PlanningEpochId(u128)` in `bcinr-mfw-ir` is explicitly designed as a 128-bit width value to hold an externally supplied globally-unique nonce, avoiding the need for stateful monotonic counters or runtime randomness.
- **Pure Math Arguments:** Deterministic pseudo-random number generation (PRNG) functions take their initial seed or state strictly as explicit input arguments (e.g., `val` and `aux`). The output randomness is fundamentally a pure function of these inputs.

## 2. $CC=1$ Branchless Pseudo-Randomness (PRNGs)

Once a seed or initial state is provided, the hot path produces subsequent pseudo-random values through mathematically pure, constant-time algorithms that contain exactly zero branches.

The `bcinr-logic` crate provides several branchless algorithmic primitives:
- **`pcg_random_u64`**: A branchless implementation of the Permuted Congruential Generator (PCG), utilizing the RXS-M-XS-64 permutation. The state transition relies entirely on constant-time bitwise operations (`wrapping_add`, `wrapping_mul`, bitshifts, XOR).
- **`random_permutation_fixed_seed`**: A fixed-seed pseudo-random permutation (bijection) built using the SplitMix64 finalizer logic seeded with the golden-ratio constant.
- **`poisson_noise_branchless` & `gaussian_noise_box_muller`**: Branchless distributional noise generators. For instance, `poisson_noise_branchless` calculates success rates by interpreting SplitMix64 outputs as independent Bernoulli trials across 64 bit-lanes, using a masked `.count_ones()` to tally successes in $O(1)$ constant time.

## 3. Cryptographic Mixing with `ChaChaSponge`

For deterministic substrate receipts requiring cryptographically secure mixing (e.g., combining and hashing causal horizons or state digests in `bcinr-mfw-ir`), BCINR utilizes a branchless ChaCha permutation.

- **`ChaChaSponge`**: Implemented in `crates/bcinr-logic/src/patterns/chacha_sponge.rs`, this component handles fixed 8-round (ChaCha8) permutations natively. 
- **Substrate Compliance**: The sponge runs completely branchlessly. By explicitly evaluating quarter-rounds with deterministic `.rotate_left()`, `.wrapping_add()`, and `^` operations, the entire hashing process avoids control flow. The full ChaCha8 permutation executes well within the Substrate's T1 aggregate latency budget (≤ 400 ns), requires 0 allocations, and passes the strict PhD gate.

## 4. Executable Proofs and Replayability

By extracting entropy generation from the authoritative execution and treating all randomness as a purely functional transformation of injected state, BCINR preserves reproducible process replay. If an execution is replayed with the exact same initial seed or nonce, the bit-for-bit identical PRNG sequences are generated.

Because the execution trace (and cycle count) never branches based on the generated bits, the entire architecture flawlessly complies with the Radon Law ($CC=1$) while supporting complex random sampling and stochastic noise operations.
