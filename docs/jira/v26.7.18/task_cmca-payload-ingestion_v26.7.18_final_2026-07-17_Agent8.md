# Task: CMCA Payload Ingestion Parser

## Metadata
* **Type:** Task
* **Subproject:** bcinr
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

## Missing Consequence (The Gap)
The BCINR implementation currently does not have a deterministic parser for the strict fixed-point numeric schema of the CMCA generated payload.

## Evidence Requirement
A parser function that correctly reconstructs the fixed-point structs from the canonical representation, validated by a passing integration test.

## Bounded Construction
Create a deserialization routine in `bcinr-core`. Implement strict deserialization utilizing `SignedFixed::from_value_bits()` that explicitly aligns with the CMCA artifact schema.

## Refusal/Negative Fixture
The parser must return a typed rejection (`UnsupportedSchema` or `MalformedPayload`) if the payload version or checksum does not strictly match the expected artifact schema.
