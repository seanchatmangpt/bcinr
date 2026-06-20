//! Pattern IR: game patterns expressed as data.
//!
//! The IR is the single data model that (a) the ggen ontology (`ggen/schema/patterns.ttl`)
//! mirrors, (b) the generated [`crate::patterns::PATTERN_REGISTRY`] is built from, and
//! (c) [`crate::verify`] iterates over for self-checks. Keeping patterns as data lets the
//! whole catalog be documented, validated, and cross-referenced uniformly.
//!
//! # Design
//!
//! Every type in this module is `Copy + PartialEq + Eq + Hash` so patterns can be used as
//! map keys, stored in hash-sets for de-duplication, or compared cheaply without cloning.
//! [`PatternId`] additionally implements `Ord` because patterns are ordered by their stable
//! numeric index and sorted registry lookups are a common access pattern.
//!
//! All [`core::fmt::Display`] implementations use `core::fmt` (no `std`) so the IR is
//! usable from `no_std` targets and embedded runtimes.

/// Stable numeric id for a pattern (mirrors the `w4g:id` of the TTL individual).
///
/// Ids are assigned at codegen time and are stable across releases — once a pattern is
/// assigned id `N` it keeps that id forever. The sentinel [`PatternId::INVALID`]
/// (`u16::MAX`) is reserved and must never appear in the registry.
///
/// `PatternId` implements `Ord` so registries can be kept in sorted order for binary
/// search, and `Hash` so ids can be used as map keys without wrapping.
///
/// # Examples
///
/// ```
/// use wasm4games::ir::PatternId;
///
/// let id = PatternId::new(7);
/// assert!(id.is_valid());
/// assert_eq!(id.to_string(), "pattern#7");
/// assert!(PatternId::INVALID > PatternId::MIN);
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct PatternId(pub u16);

impl PatternId {
    /// The smallest valid pattern id (id `0`).
    pub const MIN: Self = Self(0);

    /// The sentinel "invalid / absent" id (`u16::MAX`).
    ///
    /// No real pattern may carry this id. Use [`is_valid`](Self::is_valid) to guard
    /// against sentinel values before indexing into the registry.
    pub const INVALID: Self = Self(u16::MAX);

    /// Construct a [`PatternId`] from a raw `u16` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::PatternId;
    /// assert_eq!(PatternId::new(42).raw(), 42);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(code: u16) -> Self {
        Self(code)
    }

    /// Return the raw `u16` code.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::PatternId;
    /// assert_eq!(PatternId::new(5).raw(), 5);
    /// ```
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u16 {
        self.0
    }

    /// Return `true` if this id is a real pattern id (not the [`INVALID`](Self::INVALID)
    /// sentinel).
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::PatternId;
    /// assert!(PatternId::new(0).is_valid());
    /// assert!(!PatternId::INVALID.is_valid());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_valid(self) -> bool {
        self.0 != u16::MAX
    }
}

impl core::fmt::Display for PatternId {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "pattern#{}", self.0)
    }
}

/// The branchless lowering a kernel reduces to. Drives template choice in ggen and the
/// lowering documented in each generated kernel.
///
/// Each variant names a family of branchless primitives from `bcinr_logic`:
///
/// | Variant      | `bcinr_logic` module          |
/// |--------------|-------------------------------|
/// | `Lut`        | lookup tables                 |
/// | `Mask`       | `mask::select_u32` family     |
/// | `Saturating` | `int::add_sat_u8` family      |
/// | `Bitset`     | `bitset::*`                   |
/// | `Dfa`        | `dfa::*`                      |
/// | `Network`    | `network::*` (Benes networks) |
/// | `Receipt`    | rolling-hash chain            |
///
/// # Examples
///
/// ```
/// use wasm4games::ir::LoweringKind;
/// assert_eq!(LoweringKind::Mask.to_string(), "Mask");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum LoweringKind {
    /// Lookup-table driven: state transitions encoded in a flat `u8` table.
    Lut,
    /// Mask / branchless-select driven: uses `mask::select_u32` family.
    Mask,
    /// Saturating arithmetic driven: uses `int::add_sat_u8` / `int::sub_sat_u8`.
    Saturating,
    /// Bitset / popcount driven: uses `bitset::*` primitives.
    Bitset,
    /// Deterministic finite automaton driven: uses `dfa::*` primitives.
    Dfa,
    /// Sorting / permutation network driven: uses `network::*` (Benes) primitives.
    Network,
    /// Receipt / rolling-hash driven: seals a chain of evidence tokens.
    Receipt,
}

impl core::fmt::Display for LoweringKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let s = match self {
            LoweringKind::Lut => "Lut",
            LoweringKind::Mask => "Mask",
            LoweringKind::Saturating => "Saturating",
            LoweringKind::Bitset => "Bitset",
            LoweringKind::Dfa => "Dfa",
            LoweringKind::Network => "Network",
            LoweringKind::Receipt => "Receipt",
        };
        f.write_str(s)
    }
}

/// An object kind an event links to (OCEL object-centricity).
///
/// In the Object-Centric Event Log (OCEL) model each event is linked to one or more
/// objects of typed kinds. An `ObjectKind` pairs a stable numeric `code` with a
/// human-readable `name` for documentation and tracing.
///
/// `ObjectKind` implements `Hash` so it can be stored in hash-sets and used as a map key.
///
/// # Examples
///
/// ```
/// use wasm4games::ir::ObjectKind;
///
/// let kind = ObjectKind::new(1, "player");
/// assert_eq!(kind.to_string(), "object[1:player]");
/// assert_eq!(kind.code(), 1);
/// assert_eq!(kind.name(), "player");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct ObjectKind {
    /// Stable numeric object-type code.
    pub code: u16,
    /// Human-readable snake_case name.
    pub name: &'static str,
}

impl ObjectKind {
    /// Construct an [`ObjectKind`] from a numeric code and a static name.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::ObjectKind;
    /// let k = ObjectKind::new(42, "npc");
    /// assert_eq!(k.code, 42);
    /// assert_eq!(k.name, "npc");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(code: u16, name: &'static str) -> Self {
        Self { code, name }
    }

    /// Return the stable numeric type code.
    #[inline]
    #[must_use]
    pub const fn code(self) -> u16 {
        self.code
    }

    /// Return the human-readable snake_case name.
    #[inline]
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl core::fmt::Display for ObjectKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "object[{}:{}]", self.code, self.name)
    }
}

/// An event kind a pattern emits.
///
/// Event kinds are the typed activity labels of the OCEL trace. Each `EventKind` pairs
/// a stable numeric `code` with a PascalCase `name` (e.g. `"PlayerSpawned"`). The code
/// is stable across releases; the name is used for human-readable logging and OTEL spans.
///
/// `EventKind` implements `Hash` so it can be stored in hash-sets and used as a map key.
///
/// # Examples
///
/// ```
/// use wasm4games::ir::EventKind;
///
/// let kind = EventKind::new(3, "PlayerSpawned");
/// assert_eq!(kind.to_string(), "event[3:PlayerSpawned]");
/// assert_eq!(kind.code(), 3);
/// assert_eq!(kind.name(), "PlayerSpawned");
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct EventKind {
    /// Stable numeric event-type code.
    pub code: u16,
    /// Human-readable PascalCase activity name.
    pub name: &'static str,
}

impl EventKind {
    /// Construct an [`EventKind`] from a numeric code and a static activity name.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::EventKind;
    /// let k = EventKind::new(1, "EnemyDefeated");
    /// assert_eq!(k.code, 1);
    /// assert_eq!(k.name, "EnemyDefeated");
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(code: u16, name: &'static str) -> Self {
        Self { code, name }
    }

    /// Return the stable numeric event-type code.
    #[inline]
    #[must_use]
    pub const fn code(self) -> u16 {
        self.code
    }

    /// Return the human-readable PascalCase activity name.
    #[inline]
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }
}

impl core::fmt::Display for EventKind {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "event[{}:{}]", self.code, self.name)
    }
}

/// The admissibility contract of a pattern, mirrored from wasm4pm-compat semantics.
///
/// An `AdmissionRule` encodes two things:
/// 1. The minimum [`crate::class::status`] code a trace must carry to be eligible for the
///    pattern's kernel (`required_status`).
/// 2. The status code the kernel emits when a violation is detected (`refusal_status`).
///
/// Both codes live in the byte-class admission lattice defined in
/// [`crate::class::status`].
///
/// # Invariants
///
/// - `required_status < crate::class::status::COUNT`
/// - `refusal_status < crate::class::status::COUNT`
///
/// # Examples
///
/// ```
/// use wasm4games::ir::AdmissionRule;
/// use wasm4games::class::status;
///
/// let rule = AdmissionRule::new(status::PARTIAL, status::REFUSED);
/// assert_eq!(rule.to_string(), "admit(min=2,refuse=7)");
/// assert!(rule.is_admitted(status::ADMITTED));
/// assert!(!rule.is_admitted(status::UNKNOWN));
/// ```
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct AdmissionRule {
    /// Minimum [`crate::class::status`] code required to be admissible.
    pub required_status: u8,
    /// Status code emitted when the rule is violated.
    pub refusal_status: u8,
}

impl AdmissionRule {
    /// Construct an [`AdmissionRule`] from a required status and a refusal status.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::AdmissionRule;
    /// use wasm4games::class::status;
    ///
    /// let rule = AdmissionRule::new(status::ADMITTED, status::REFUSED);
    /// assert_eq!(rule.required_status, status::ADMITTED);
    /// assert_eq!(rule.refusal_status, status::REFUSED);
    /// ```
    #[inline]
    #[must_use]
    pub const fn new(required_status: u8, refusal_status: u8) -> Self {
        Self {
            required_status,
            refusal_status,
        }
    }

    /// Return the minimum status code required for admission.
    #[inline]
    #[must_use]
    pub const fn required_status(self) -> u8 {
        self.required_status
    }

    /// Return the status code emitted on a violation.
    #[inline]
    #[must_use]
    pub const fn refusal_status(self) -> u8 {
        self.refusal_status
    }

    /// Return `true` when the supplied `status` code satisfies the admission requirement.
    ///
    /// The check is a purely numeric comparison: `status >= self.required_status`.
    /// It is branchless at the call site because it compiles to a single `cmp`/`setae`.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::AdmissionRule;
    /// use wasm4games::class::status;
    ///
    /// let rule = AdmissionRule::new(status::PARTIAL, status::REFUSED);
    /// assert!(rule.is_admitted(status::PARTIAL));
    /// assert!(rule.is_admitted(status::ADMITTED));
    /// assert!(!rule.is_admitted(status::UNKNOWN));
    /// assert!(!rule.is_admitted(status::BLOCKED));
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_admitted(self, status: u8) -> bool {
        status >= self.required_status
    }
}

impl core::fmt::Display for AdmissionRule {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "admit(min={},refuse={})",
            self.required_status, self.refusal_status
        )
    }
}

impl Default for AdmissionRule {
    /// Returns the most permissive admission rule: requires `UNKNOWN` (0) and refuses
    /// with `REFUSED` (7). Equivalent to "always admit; on violation emit code 7."
    #[inline]
    fn default() -> Self {
        use crate::class::status;
        Self::new(status::UNKNOWN, status::REFUSED)
    }
}

/// A pattern as data: enough to document it, self-check it, and cross-reference the TTL.
///
/// `PatternSpec` is the central IR node. One instance exists per pattern in the registry
/// ([`crate::patterns::PATTERN_REGISTRY`]). It is `Copy` because it contains only
/// primitive values and `'static` references, making it zero-cost to pass around and
/// store in `const` items.
///
/// # Invariants
///
/// - `id` is unique across the registry and not [`PatternId::INVALID`]
/// - `name` is non-empty and matches the generated kernel file stem
/// - `state_card` > 0 (at least one state in the byte-class alphabet)
/// - `otel_span` matches the `crate::evidence::otel` constant for this pattern
///
/// # Examples
///
/// ```
/// use wasm4games::ir::{PatternSpec, PatternId, LoweringKind, EventKind, AdmissionRule};
///
/// const SPEC: PatternSpec = PatternSpec {
///     id: PatternId::new(1),
///     name: "coin_collect",
///     lowering: LoweringKind::Mask,
///     state_card: 4,
///     event: EventKind::new(1, "CoinCollected"),
///     objects: &[],
///     admission: AdmissionRule::new(0, 7),
///     otel_span: 0x0001,
/// };
/// assert!(SPEC.id.is_valid());
/// assert_eq!(SPEC.id.to_string(), "pattern#1");
/// assert!(SPEC.is_valid());
/// ```
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

impl PatternSpec {
    /// Return `true` when the spec passes basic internal consistency checks:
    ///
    /// - `id` is valid (not the [`PatternId::INVALID`] sentinel)
    /// - `name` is non-empty
    /// - `state_card` > 0
    ///
    /// This is a lightweight runtime sanity-check. It does not verify uniqueness within
    /// the registry — that is the responsibility of [`crate::verify`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::{PatternSpec, PatternId, LoweringKind, EventKind, AdmissionRule};
    ///
    /// let valid = PatternSpec {
    ///     id: PatternId::new(1),
    ///     name: "ok",
    ///     lowering: LoweringKind::Lut,
    ///     state_card: 1,
    ///     event: EventKind::new(0, "Noop"),
    ///     objects: &[],
    ///     admission: AdmissionRule::new(0, 7),
    ///     otel_span: 0,
    /// };
    /// assert!(valid.is_valid());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        self.id.is_valid() && !self.name.is_empty() && self.state_card > 0
    }

    /// Return `true` when `status` satisfies this pattern's admission rule.
    ///
    /// Delegates to [`AdmissionRule::is_admitted`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::{PatternSpec, PatternId, LoweringKind, EventKind, AdmissionRule};
    /// use wasm4games::class::status;
    ///
    /// let spec = PatternSpec {
    ///     id: PatternId::new(1),
    ///     name: "demo",
    ///     lowering: LoweringKind::Mask,
    ///     state_card: 1,
    ///     event: EventKind::new(0, "E"),
    ///     objects: &[],
    ///     admission: AdmissionRule::new(status::PARTIAL, status::REFUSED),
    ///     otel_span: 0,
    /// };
    /// assert!(spec.is_admitted(status::PARTIAL));
    /// assert!(!spec.is_admitted(status::UNKNOWN));
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_admitted(&self, status: u8) -> bool {
        self.admission.is_admitted(status)
    }

    /// Return the number of object kinds linked to this pattern's event.
    ///
    /// # Examples
    ///
    /// ```
    /// use wasm4games::ir::{PatternSpec, PatternId, LoweringKind, EventKind, AdmissionRule, ObjectKind};
    ///
    /// static OBJS: &[ObjectKind] = &[ObjectKind { code: 1, name: "player" }];
    /// let spec = PatternSpec {
    ///     id: PatternId::new(1),
    ///     name: "demo",
    ///     lowering: LoweringKind::Mask,
    ///     state_card: 1,
    ///     event: EventKind::new(0, "E"),
    ///     objects: OBJS,
    ///     admission: AdmissionRule::new(0, 7),
    ///     otel_span: 0,
    /// };
    /// assert_eq!(spec.object_count(), 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn object_count(&self) -> usize {
        self.objects.len()
    }
}

impl core::fmt::Display for PatternSpec {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "PatternSpec({}, {}, {})",
            self.id, self.name, self.lowering
        )
    }
}

#[cfg(test)]
mod tests {
    extern crate alloc;
    use super::*;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;

    #[test]
    fn pattern_id_behavior() {
        // Display
        assert_eq!(PatternId::new(0).to_string(), "pattern#0");
        assert_eq!(PatternId::new(42).to_string(), "pattern#42");
        assert_eq!(PatternId::INVALID.to_string(), "pattern#65535");

        // Ord + sort
        assert!(PatternId::MIN < PatternId::new(1));
        assert!(PatternId::new(1) < PatternId::new(2));
        assert!(PatternId::new(99) < PatternId::INVALID);
        let mut ids = [PatternId::new(3), PatternId::new(1), PatternId::new(2)];
        ids.sort();
        assert_eq!(
            ids,
            [PatternId::new(1), PatternId::new(2), PatternId::new(3)]
        );

        // validity / sentinels / default / raw roundtrip
        assert!(PatternId::MIN.is_valid());
        assert!(PatternId::new(1000).is_valid());
        assert!(!PatternId::INVALID.is_valid());
        assert_eq!(PatternId::MIN.raw(), 0);
        assert_eq!(PatternId::INVALID.raw(), u16::MAX);
        assert_eq!(PatternId::default(), PatternId::MIN);
        for v in [0u16, 1, 100, 999, u16::MAX - 1] {
            assert_eq!(PatternId::new(v).raw(), v);
        }

        // usable as BTreeMap key
        let mut map = BTreeMap::new();
        map.insert(PatternId::new(1), "first");
        map.insert(PatternId::new(2), "second");
        assert_eq!(map[&PatternId::new(1)], "first");
        assert_eq!(map[&PatternId::new(2)], "second");
    }

    #[test]
    fn lowering_object_event_kind_behavior() {
        // LoweringKind Display + Eq
        let cases: &[(LoweringKind, &str)] = &[
            (LoweringKind::Lut, "Lut"),
            (LoweringKind::Mask, "Mask"),
            (LoweringKind::Saturating, "Saturating"),
            (LoweringKind::Bitset, "Bitset"),
            (LoweringKind::Dfa, "Dfa"),
            (LoweringKind::Network, "Network"),
            (LoweringKind::Receipt, "Receipt"),
        ];
        for &(ref lk, name) in cases {
            assert_eq!(lk.to_string(), name);
        }
        assert_eq!(LoweringKind::Mask, LoweringKind::Mask);
        assert_ne!(LoweringKind::Lut, LoweringKind::Dfa);

        // ObjectKind Display + accessors + BTreeMap key
        let k = ObjectKind::new(1, "player");
        assert_eq!(k.to_string(), "object[1:player]");
        assert_eq!(ObjectKind::new(7, "npc").code(), 7);
        assert_eq!(ObjectKind::new(7, "npc").name(), "npc");
        let mut map = BTreeMap::new();
        map.insert(ObjectKind::new(1, "player"), "p1");
        assert_eq!(map[&ObjectKind::new(1, "player")], "p1");

        // EventKind Display + accessors + BTreeMap key
        let k = EventKind::new(3, "PlayerSpawned");
        assert_eq!(k.to_string(), "event[3:PlayerSpawned]");
        assert_eq!(EventKind::new(99, "BossDefeated").code(), 99);
        assert_eq!(EventKind::new(99, "BossDefeated").name(), "BossDefeated");
        let mut map = BTreeMap::new();
        map.insert(EventKind::new(3, "Foo"), "handler");
        assert_eq!(map[&EventKind::new(3, "Foo")], "handler");
    }

    #[test]
    fn admission_rule_behavior() {
        use crate::class::status;

        // Display
        assert_eq!(
            AdmissionRule::new(status::PARTIAL, status::REFUSED).to_string(),
            "admit(min=2,refuse=7)"
        );

        // is_admitted
        let rule = AdmissionRule::new(status::PARTIAL, status::REFUSED);
        assert!(!rule.is_admitted(status::UNKNOWN));
        assert!(!rule.is_admitted(status::BLOCKED));
        assert!(rule.is_admitted(status::PARTIAL));
        assert!(rule.is_admitted(status::ADMITTED));
        assert!(rule.is_admitted(status::REFUSED));
        assert!(rule.is_admitted(status::RESIDUAL));

        // default is most permissive
        let rule = AdmissionRule::default();
        assert!(rule.is_admitted(status::UNKNOWN));
        assert!(rule.is_admitted(status::PARTIAL));
        assert_eq!(rule.required_status, status::UNKNOWN);
        assert_eq!(rule.refusal_status, status::REFUSED);

        // accessors + roundtrip
        let rule = AdmissionRule::new(status::ADMITTED, status::REFUSED);
        assert_eq!(rule.required_status(), status::ADMITTED);
        assert_eq!(rule.refusal_status(), status::REFUSED);
        let rule = AdmissionRule::new(3, 7);
        assert_eq!((rule.required_status, rule.refusal_status), (3, 7));
    }

    fn make_spec(id: u16, name: &'static str, card: u8) -> PatternSpec {
        PatternSpec {
            id: PatternId::new(id),
            name,
            lowering: LoweringKind::Mask,
            state_card: card,
            event: EventKind::new(0, "E"),
            objects: &[],
            admission: AdmissionRule::new(0, 7),
            otel_span: 0,
        }
    }

    #[test]
    fn pattern_spec_behavior() {
        use crate::class::status;

        // is_valid: valid / invalid id / empty name / zero card
        assert!(make_spec(1, "coin_collect", 4).is_valid());
        assert!(make_spec(0, "zero_id", 1).is_valid());
        assert!(!make_spec(u16::MAX, "bad", 1).is_valid());
        assert!(!make_spec(1, "", 1).is_valid());
        assert!(!make_spec(1, "bad_card", 0).is_valid());

        // Display
        let spec = PatternSpec {
            id: PatternId::new(5),
            name: "demo",
            lowering: LoweringKind::Network,
            state_card: 2,
            event: EventKind::new(0, "Demo"),
            objects: &[],
            admission: AdmissionRule::new(0, 7),
            otel_span: 0,
        };
        assert_eq!(spec.to_string(), "PatternSpec(pattern#5, demo, Network)");

        // object_count
        static OBJS: &[ObjectKind] = &[
            ObjectKind {
                code: 1,
                name: "player",
            },
            ObjectKind {
                code: 2,
                name: "item",
            },
        ];
        let spec = PatternSpec {
            id: PatternId::new(1),
            name: "multi_obj",
            lowering: LoweringKind::Mask,
            state_card: 1,
            event: EventKind::new(1, "Collected"),
            objects: OBJS,
            admission: AdmissionRule::new(0, 7),
            otel_span: 0,
        };
        assert_eq!(spec.object_count(), 2);

        // is_admitted delegates to AdmissionRule
        let spec = PatternSpec {
            id: PatternId::new(1),
            name: "s",
            lowering: LoweringKind::Mask,
            state_card: 1,
            event: EventKind::new(0, "E"),
            objects: &[],
            admission: AdmissionRule::new(status::PARTIAL, status::REFUSED),
            otel_span: 0,
        };
        assert!(spec.is_admitted(status::PARTIAL));
        assert!(spec.is_admitted(status::ADMITTED));
        assert!(!spec.is_admitted(status::UNKNOWN));
        assert!(!spec.is_admitted(status::BLOCKED));
    }
}
