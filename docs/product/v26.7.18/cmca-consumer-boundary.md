# Product Requirements Document: BCINR CMCA Consumer Boundary

## Context
**Coordinate:** v26.7.18
**Subproject:** `bcinr`
**Standing:** `UNKNOWN` (Candidate Declaration)

## Mission
To manufacture the consequence of deterministic CMCA payload ingestion into the `bcinr` subproject while strictly preserving the distinction between semantic production and runtime execution.

## Zero-Loss Information & Combinatorial Maximalism
The product requires the complete, exact transfer of the fixed-point numeric schema from the generated CMCA artifact to the consumer memory space.
* The system must model the entire domain of valid CMCA payloads deterministically.
* Combinatorial exhaustion of the structural space dictates that any malformed or untrusted payload state must be explicitly covered and safely rejected without losing diagnostic context.

## Exact Capabilities and Refusals
* **Capability:** Reconstruct fixed-point structs from canonical bit-patterns utilizing `SignedFixed::from_value_bits()` without an underlying OS (`no-std` compatible).
* **Refusal:** Return exact typed outcomes (`UnsupportedSchema` or `MalformedPayload`) if the payload version or checksum does not strictly match the expected coordinate.
