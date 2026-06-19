//! Pattern IR: game patterns expressed as data.
//!
//! The IR is the single data model that (a) the ggen ontology (`ggen/schema/patterns.ttl`)
//! mirrors, (b) the generated [`crate::patterns::PATTERN_REGISTRY`] is built from, and
//! (c) [`crate::verify`] iterates over for self-checks. Keeping patterns as data lets the
//! whole catalog be documented, validated, and cross-referenced uniformly.

/// Stable numeric id for a pattern (mirrors the `w4g:id` of the TTL individual).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PatternId(pub u16);

/// The branchless lowering a kernel reduces to. Drives template choice in ggen and the
/// lowering documented in each generated kernel.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum LoweringKind {
    /// Lookup-table driven.
    Lut,
    /// Mask / branchless-select driven.
    Mask,
    /// Saturating arithmetic driven.
    Saturating,
    /// Bitset / popcount driven.
    Bitset,
    /// Deterministic finite automaton driven.
    Dfa,
    /// Sorting / permutation network driven.
    Network,
    /// Receipt / rolling-hash driven.
    Receipt,
}

/// An object kind an event links to (OCEL object-centricity).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ObjectKind {
    /// Stable numeric object-type code.
    pub code: u16,
    /// Human-readable snake_case name.
    pub name: &'static str,
}

/// An event kind a pattern emits.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct EventKind {
    /// Stable numeric event-type code.
    pub code: u16,
    /// Human-readable PascalCase activity name.
    pub name: &'static str,
}

/// The admissibility contract of a pattern, mirrored from wasm4pm-compat semantics.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AdmissionRule {
    /// Minimum [`crate::class::status`] code required to be admissible.
    pub required_status: u8,
    /// Status code emitted when the rule is violated.
    pub refusal_status: u8,
}

/// A pattern as data: enough to document it, self-check it, and cross-reference the TTL.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct PatternSpec {
    /// Stable id.
    pub id: PatternId,
    /// snake_case kernel function name (matches the generated file stem).
    pub name: &'static str,
    /// Branchless lowering used by the kernel.
    pub lowering: LoweringKind,
    /// Byte-class cardinality of the kernel's primary state.
    pub state_card: u8,
    /// The event the pattern emits.
    pub event: EventKind,
    /// The object kinds the event links to.
    pub objects: &'static [ObjectKind],
    /// The admissibility contract.
    pub admission: AdmissionRule,
    /// The OTEL span code emitted at runtime (matches `crate::evidence::otel`).
    pub otel_span: u16,
}
