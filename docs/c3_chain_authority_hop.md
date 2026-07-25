# The C3 (Control, Certification, Commit) Chain in BCINR

The **C3 Chain (Control, Certification, Commit)** in BCINR is a deterministic, linear sequence of validations that governs state admission and mode transitions. It enforces that every change to the persistent authoritative state first passes a sequence of immutable verification steps—preventing caller bypasses, incomplete audits, or "fake" state injections. 

This chain is structurally enforced by **Authority Hops**, modeled closely around Rust's typestate pattern combined with fixed-point arithmetic and branchless, allocation-free execution (as dictated by the Radon Law, $CC=1$). 

## The Linear Sequence of Validations (The Authority Hops)

The C3 chain is divided into specialized modules representing each "hop." A downstream hop strictly requires the cryptographic/sealed output of the upstream hop as proof.

1. **Hop 1: Mode-change proposal (`proposal.rs`) — Control** 
   The initial step where telemetry is evaluated. It binds the proposed control delta, mode digests, and round identity into a sealed `ModeProposal`. It can only be constructed by the Observatory's lawful evaluation path.
2. **Hop 2: Shadow execution (`shadow.rs`)** 
   Executes the candidate mode against the current mode to compute comparison values (deltas) without making any actual persistent writes. It strictly adheres to the "SELECT is never DO" law and emits a `ShadowExecutionReceipt`.
3. **Hop 3: Jump analysis (`jump.rs`)** 
   Classifies the shadow comparison into a specific jump category (`PolicyJump`, `FixedPointStateJump`, or `SwitchingDisturbance`). It produces a sealed `JumpAnalysisReceipt`.
4. **Hop 4: Stability candidate (`stability.rs`)** 
   Takes the jump analysis and verifies the static domination law $G d \leq (1 - \delta) d$. The runtime never *discovers* the theorem here, it only *verifies* it. Only if the contraction margin holds does it yield a `StabilityCandidate`.
5. **Hop 5: Certificate sealing and dwell (`certification.rs`) — Certification**
   The sole gateway that mints a `CertificateReceipt`. This hop is highly rigorous and verifies two main components:
   - **Witness Verification:** Independently recomputes the domination witness from the `StabilityCandidate`'s fields.
   - **Domain Bindings:** Explicitly verifies 11 domain-specific bindings (e.g., admitted graph, generated payload, kernel identity, numeric profile, pricing law, etc.). Any single mismatch refuses the seal. There is no partial admission. 
   - **Temporal Gating:** Enforces a dwell law (`DwellSatisfied`) proving a sufficient amount of time elapsed for the specific round/transition.
6. **Hop 6: Mode switch (`mode_switch.rs`) — Commit**
   The final actuation step. It atomically applies the mode switch, providing an independent test guarantee that a rejected transition leaves every persistent byte bit-for-bit unchanged from its pre-attempt snapshot.

## How it structurally enforces inescapability prior to admission

The C3 architecture relies on **typestate and authority separation** to make bypasses mechanically impossible:

1. **Opaque Types and "No Alternate Constructors":** Validations emit opaque evidence types (e.g., `DwellSatisfied`, `StabilityCandidate`, `CertificateReceipt`) with completely private fields. There are no public constructors, no `#[derive(Default)]`, and no `serde` deserialization paths allowed. The only way to get a `CertificateReceipt` is to successfully execute the exact logic in `seal_certificate()`.
2. **Cryptographic/Digest Binding:** Every hop incorporates a fixed avalanche mix (`mix64`) of the data into a sealed digest. Hop 5 binds all domain identities to the certificate digest. If a caller tampers with the round ID or the payload digest in transit, the digest mismatch immediately refuses admission.
3. **SELECT is never DO:** Observing, proposing, and verifying (SELECT) are structurally separated from actuating (DO). A function that reads telemetry or evaluates stability is forbidden from mutating persistent state. It can only emit a token that the `mode_switch` (Commit) hop will consume.
4. **Proofs over Booleans:** Temporal gating (dwell) is never a bare `bool` that an upstream caller can supply as `true`. It is an opaque proof token (`DwellSatisfied`) bound directly to the round and transition identities, meaning a proof from an old transition cannot be re-used.

By intertwining strict module privacy, branchless typestates, and continuous cryptographic binding, the C3 chain establishes an unforgeable, un-bypassable path from Control to Commit.
