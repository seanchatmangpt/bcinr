# Epic: BCINR CMCA Consumer Boundary Integration

## Metadata
* **Type:** Epic
* **Subproject:** bcinr
* **Coordinate:** v26.7.18
* **Standing:** `DRAFT`

## Missing Consequence (The Gap)
BCINR currently lacks the strict mechanical boundary required to consume generated CMCA payloads safely without leaking semantic tooling dependencies into the production environment. The consumer must strictly isolate the payload ingestion.

## Evidence Requirement
A verifiable CMCA artifact consumer module in the BCINR codebase, along with a test proving that no semantic tools (e.g., RDF engines or Python generation scripts) are transitively depended upon during payload ingestion.

## Bounded Construction
Implement a pure, no-std compatible fixed-point consumer API within `bcinr-core`. Parse the pre-generated CMCA payload directly. Do not include any semantic resolution logic.

## Refusal/Negative Fixture
The build must cleanly refuse to compile if any semantic producer crate (e.g., RDF parsing logic) is imported.
