# Data Models and Structs in BCINR

The `bcinr` repository operates as a deterministic, branchless compute substrate. As a result, it does not use traditional database schemas, ORMs, or SQL migrations. All data models are defined via fixed-width structs and proof tokens in Rust to guarantee allocation-free, branchless execution on the hot path (as mandated by `$CC=1` and `#![no_std]` constraints).

## 1. Core Mathematical Primitives (Fixed-Point Arithmetic)
Instead of floating-point numbers, state is tracked using deterministic Q16.16 fixed-point representations. These types encapsulate both their numeric value and any fault tracking.
- **`NonNegativeFixed`**: A non-negative Q16.16 fixed-point value with a sealed representation (magnitude and fault set travel together).
- **`SignedFixed`**: The signed equivalent for fixed-point arithmetic.
- **`CanonicalMask`**: A boolean mask (`u32`) for branchless logic operations (`select(mask, a, b)`).
- **`NumericFaultSet`**: A bitset (`u32`) indicating mathematical faults (e.g., overflow, underflow).

## 2. Cryptographic and Typestate Proof Tokens (Certificates)
To avoid speculative mutation and branching, operations require strictly-typed ZSTs (Zero-Sized Types) or tokens containing cryptographic digests. These act as proofs that preconditions have been met.
- **`AdmittedControlState`**: Proof token certifying that the control state has been admitted.
- **`CertificateReceipt`**, **`EnvelopeReceipt`**, **`OutcomeReceipt`**: Proof tokens certifying receipt of valid security certificates, envelopes, and outcomes.
- **`AdaptiveUpdate<Mode>`**: A proof token certifying that an adaptive update is authorized.
- **`CertifiedLearning`** & **`CertifiedSelectionOnly`**: Marker ZST structs used to enforce control flow mode constraints at compile-time.

## 3. State and General Data Models
Data models are packed structs optimized for fixed bounded memory access:
- **`PackedSemanticState`**: Represents the core semantic state of an entity (e.g., `id: u32`, `factors: [NonNegativeFixed; F]`).
- **`LensSpec`**: Contains an identifier (`id: u32`) and parameters (`q: SignedFixed`).
- **`ModeState`**: Persistent mode-state acting as a surrogate for orchestration (tracks `mode_digest: u64` and `generation: u64`).
- **`CertifiedModeSwitch`**: Sealed proof that a mode switch was prepared correctly.
- **`ActuationEvidence`**: Detailed record of a mode switch attempt and its outcome (`Applied` or `Refused(ModeSwitchRefusal)`).

## 4. Artifact & Manifest Structures
Metadata mapping generated logic to its profiles is stored statically.
- **`GeneratedManifest`**, **`ManifestDigests`**, **`ManifestDimensions`**, **`NumericProfile`**: Structs reflecting static characteristics and bounds of the generated models.

## Database Schemas
There are **no database schemas** (`.sql`, `diesel`, etc.) within this repository. The system manages memory and state exclusively via fixed-sized configurations and typestates suitable for high-assurance embedded or zero-allocation systems.
