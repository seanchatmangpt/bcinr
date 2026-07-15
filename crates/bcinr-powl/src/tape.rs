//! POWL v2 op tape — flat, cache-line-aligned, `no_std`-compatible.
//!
//! The top-level module preserves the legacy API used by `compiler.rs` and
//! `scheduler.rs` (with `OpKind::Atom/Join/XorDispatch/LoopRedo`, the `alloc`
//! method, and the `entry_mask`/`branch_mask`/`kind` fields).
//!
//! The new POWL v2 types (`Powl64Op` with 64-byte cache-line layout,
//! `PowlTape` with `entry_op`/`exit_op`, `LabelSlab`, `PowlTapeLarge`) live
//! in the [`v2`] submodule.

/// Discriminant for each slot in the tape.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum OpKind {
    /// Concrete activity (named atom).
    Atom = 0,
    /// Silent / tau transition — fires automatically when enabled.
    Silent = 1,
    /// XOR-dispatcher: when fired, sets exactly one branch live via choice_taken.
    XorDispatch = 2,
    /// Join point: waits for all predecessor branches (normal pred_mask semantics).
    Join = 3,
    /// Loop back-edge: redo exit that re-enables the body entry.
    LoopRedo = 4,
}

/// A single operation slot on the POWL tape (64-bit aligned, 32 bytes).
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(8))]
pub struct Powl64Op {
    /// Bitmask of slots that must be `done` before this slot is enabled.
    pub pred_mask: u64,
    /// Bitmask of slots whose `check_mask` is updated when this slot completes.
    pub succ_mask: u64,
    /// For XorDispatch: bitmask of branch-entry slots (one will be chosen).
    /// For other kinds: 0.
    pub branch_mask: u64,
    /// Kind of this slot.
    pub kind: OpKind,
    /// Index of this slot (redundant but useful for debugging).
    pub index: u8,
    /// For XorDispatch slots: number of branches.
    pub branch_count: u8,
    _pad: [u8; 5],
}

impl Powl64Op {
    pub const fn new(kind: OpKind, index: u8) -> Self {
        Self {
            pred_mask: 0,
            succ_mask: 0,
            branch_mask: 0,
            kind,
            index,
            branch_count: 0,
            _pad: [0u8; 5],
        }
    }
}

/// A compiled POWL tape: at most 64 slots.
#[derive(Clone, Debug, PartialEq)]
pub struct PowlTape {
    pub ops: [Powl64Op; 64],
    /// Number of valid slots.
    pub len: u8,
    /// Bitmask of entry-point slots (no predecessors in the DAG sense).
    pub entry_mask: u64,
}

impl PowlTape {
    pub fn new() -> Self {
        Self {
            ops: [Powl64Op::new(OpKind::Silent, 0); 64],
            len: 0,
            entry_mask: 0,
        }
    }

    /// Allocate the next slot and return its index.
    pub fn alloc(&mut self, kind: OpKind) -> Option<u8> {
        if self.len >= 64 {
            return None;
        }
        let idx = self.len;
        self.ops[idx as usize] = Powl64Op::new(kind, idx);
        self.len += 1;
        Some(idx)
    }
}

impl Default for PowlTape {
    fn default() -> Self {
        Self::new()
    }
}

// =============================================================================
// POWL v2 new types — cache-line-aligned tape with LabelSlab
// =============================================================================

/// POWL v2 op tape implementation.
///
/// All new types (`Powl64OpV2`, `PowlTapeV2`, `PowlTapeLarge`, `LabelSlab`)
/// live here.  They are `no_std`-compatible (only `core` imports) — **with
/// one exception**: [`v2::CompiledNonFace`] and [`v2::ConcurrencyGuardTable`]
/// pull in `bcinr_mfw_ir::{EventSet, Digest}`, and `bcinr-mfw-ir` itself uses
/// `std::collections::{BTreeMap, BTreeSet}` (see its `causal.rs`/
/// `concurrency.rs`), so those two types are not `no_std`-compatible. They
/// are grouped with the rest of v2 anyway because they are conceptually
/// part of the same compiled-tape output (the guard table that gates
/// concurrent firing of the ops right above them), not because they share
/// the `no_std` property.
pub mod v2 {
    use core::mem;
    use core::str;

    // =========================================================================
    // OpKind v2
    // =========================================================================

    /// The semantic kind of a single POWL v2 op.
    #[repr(u8)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum OpKind {
        /// Concrete activity (leaf node).
        Activity = 0,
        /// Silent/skip transition (τ).
        Silent = 1,
        /// XOR-choice gateway (exactly one branch taken).
        XorChoice = 3,
        /// AND-parallel gateway (all branches taken concurrently).
        Parallel = 4,
        /// Loop body / redo gateway.
        Loop = 5,
        /// Strict partial order (DAG sub-order).
        StrictPartial = 6,
        /// Choice-graph (generalised branching).
        ChoiceGraph = 7,
        /// Concurrency marker (`ctrl == u64::MAX`).
        Concur = 8,
    }

    // =========================================================================
    // Powl64Op — one cache line
    // =========================================================================

    /// A single compiled POWL v2 op, packed into one 64-byte cache line.
    ///
    /// Predecessor and successor relationships are encoded as bitmasks over the
    /// op array index.  The runtime clears bits in `pred_mask` as predecessors
    /// complete and sets bits in `succ_mask` when this op finishes.
    ///
    /// `ctrl == u64::MAX` is the concurrency marker (see [`OpKind::Concur`]).
    #[repr(C, align(64))]
    #[derive(Clone, Copy, Debug)]
    pub struct Powl64Op {
        /// Bitmask of predecessor op indices that must complete before this op.
        pub pred_mask: u64,
        /// Bitmask of successor op indices that this op activates on completion.
        pub succ_mask: u64,
        /// Control word; `u64::MAX` signals a concurrency marker.
        pub ctrl: u64,
        /// Semantic kind of this op.
        pub op_kind: OpKind,
        /// Choice-group identifier (0 = no group).
        pub choice_group: u8,
        /// Nesting depth in the POWL hierarchy.
        pub depth: u8,
        /// Number of outgoing edges (fan-out degree).
        pub fan_out: u8,
        /// Padding to reach exactly 64 bytes.
        /// Layout: 3×u64(24) + 4×u8(4) + pad(36) = 64.
        pub _pad: [u8; 36],
    }

    // Compile-time size enforcement.
    const _: () = assert!(mem::size_of::<Powl64Op>() == 64);
    const _: () = assert!(mem::align_of::<Powl64Op>() == 64);

    impl Powl64Op {
        /// Construct a silent no-op entry (usable as a const array initialiser).
        #[inline(always)]
        pub const fn silent() -> Self {
            Self {
                pred_mask: 0,
                succ_mask: 0,
                ctrl: 0,
                op_kind: OpKind::Silent,
                choice_group: 0,
                depth: 0,
                fan_out: 0,
                _pad: [0u8; 36],
            }
        }

        /// Returns `true` when this op is a concurrency marker.
        ///
        /// Implemented branchlessly via [`eq_mask_u64`].
        #[inline(always)]
        pub fn is_concur(&self) -> bool {
            eq_mask_u64(self.ctrl, u64::MAX) != 0
        }
    }

    // =========================================================================
    // eq_mask_u64
    // =========================================================================

    /// Branchless equality mask for `u64`: returns `u64::MAX` when `a == b`,
    /// `0` otherwise.  Mirrors the `eq_mask_u32` pattern in `bcinr-logic/mask`.
    ///
    /// # Hoare-logic proof
    ///
    /// Pre:  `{ a, b ∈ u64 }`
    /// Post: `{ result == u64::MAX ↔ a == b }`
    ///
    /// Let `diff = a ^ b`.
    /// - `diff == 0` ↔ `a == b`.
    /// - `(diff | diff.wrapping_neg()) >> 63` equals `1` iff `diff != 0`
    ///   (standard two's-complement zero-detection), `0` otherwise.
    /// - `nonzero_bit.wrapping_sub(1)` maps `0 → u64::MAX`, `1 → 0`. ∎
    #[inline(always)]
    pub const fn eq_mask_u64(a: u64, b: u64) -> u64 {
        let diff = a ^ b;
        let nonzero_bit = (diff | diff.wrapping_neg()) >> 63;
        nonzero_bit.wrapping_sub(1)
    }

    // =========================================================================
    // LabelSlab
    // =========================================================================

    /// A fixed-size slab for interned activity-label UTF-8 bytes.
    ///
    /// Labels are stored as `[len_lo: u8][len_hi: u8][bytes...]` entries packed
    /// sequentially.  [`intern`][LabelSlab::intern] returns the byte offset of
    /// the length prefix; [`get`][LabelSlab::get] reconstructs a `&str`.
    ///
    /// Total capacity: 1024 bytes of raw storage.
    #[derive(Debug)]
    pub struct LabelSlab {
        /// Raw packed storage: `[u16-len-le][utf8-bytes]...`
        pub data: [u8; 1024],
        /// Number of bytes currently used in `data`.
        pub len: u16,
    }

    impl LabelSlab {
        /// Construct an empty slab.
        pub const fn new() -> Self {
            Self {
                data: [0u8; 1024],
                len: 0,
            }
        }

        /// Intern `label`, returning the byte offset of its length prefix in
        /// `data`.  If the label is already present the existing offset is
        /// returned (linear-scan deduplication).
        ///
        /// Returns `u16::MAX` as a sentinel when the slab has no room.
        pub fn intern(&mut self, label: &str) -> u16 {
            let bytes = label.as_bytes();
            let label_len = bytes.len();

            // Linear scan for an existing entry.
            let mut cursor: usize = 0;
            while cursor + 2 <= self.len as usize {
                let entry_len =
                    u16::from_le_bytes([self.data[cursor], self.data[cursor + 1]]) as usize;
                let entry_start = cursor + 2;
                let entry_end = entry_start + entry_len;
                if entry_end > self.len as usize {
                    break; // truncated — stop scan
                }
                if entry_len == label_len && &self.data[entry_start..entry_end] == bytes {
                    return cursor as u16;
                }
                cursor = entry_end;
            }

            // Append new entry.
            let needed = 2 + label_len;
            let used = self.len as usize;
            if used + needed > 1024 {
                debug_assert!(false, "LabelSlab out of space");
                return u16::MAX;
            }

            let offset = used as u16;
            self.data[used] = (label_len & 0xFF) as u8;
            self.data[used + 1] = ((label_len >> 8) & 0xFF) as u8;
            self.data[used + 2..used + 2 + label_len].copy_from_slice(bytes);
            self.len = (used + needed) as u16;
            offset
        }

        /// Reconstruct the `&str` stored at `offset` (the value returned by
        /// [`intern`][Self::intern]).
        ///
        /// # Panics
        ///
        /// Panics if `offset` is out of bounds or the stored bytes are not valid
        /// UTF-8.
        pub fn get(&self, offset: u16) -> &str {
            let off = offset as usize;
            let entry_len = u16::from_le_bytes([self.data[off], self.data[off + 1]]) as usize;
            let start = off + 2;
            let end = start + entry_len;
            str::from_utf8(&self.data[start..end])
                .expect("LabelSlab: stored bytes are not valid UTF-8")
        }
    }

    impl Default for LabelSlab {
        fn default() -> Self {
            Self::new()
        }
    }

    // =========================================================================
    // PowlTape — ≤ 64 ops
    // =========================================================================

    /// A compact POWL v2 op tape for programs with ≤ 64 ops.
    ///
    /// A single `u64` bitmask is sufficient to represent the full ready-set,
    /// making all scheduling decisions branchless integer operations.
    #[derive(Debug)]
    pub struct PowlTape {
        /// The flat op array.  Valid entries are `ops[0..len]`.
        pub ops: [Powl64Op; 64],
        /// Number of valid entries.
        pub len: u8,
        /// Index of the entry op (the unique op with `pred_mask == 0`).
        pub entry_op: u8,
        /// Index of the exit op (the unique op with `succ_mask == 0`).
        pub exit_op: u8,
        /// Interned activity label storage.
        pub label_slab: LabelSlab,
    }

    impl PowlTape {
        /// Construct an empty tape.
        pub fn new() -> Self {
            Self {
                ops: [Powl64Op::silent(); 64],
                len: 0,
                entry_op: 0,
                exit_op: 0,
                label_slab: LabelSlab::new(),
            }
        }

        /// Push an op onto the tape, returning its index.
        ///
        /// Returns `None` when the tape is full (64 ops).
        #[inline]
        pub fn push(&mut self, op: Powl64Op) -> Option<u8> {
            if self.len >= 64 {
                return None;
            }
            let idx = self.len;
            self.ops[idx as usize] = op;
            self.len += 1;
            Some(idx)
        }

        /// Compute the ready-set: all ops whose `pred_mask` is zero.
        /// Returns a `u64` bitmask over op indices.
        ///
        /// Implemented branchlessly: `eq_mask_u64(pred, 0)` returns `u64::MAX`
        /// when an op is ready; the high bit is extracted and shifted to position `i`.
        #[inline]
        pub fn ready_mask(&self) -> u64 {
            let mut mask: u64 = 0;
            let n = self.len as usize;
            for i in 0..n {
                let ready = eq_mask_u64(self.ops[i].pred_mask, 0);
                mask |= ((ready >> 63) & 1) << i;
            }
            mask
        }
    }

    impl Default for PowlTape {
        fn default() -> Self {
            Self::new()
        }
    }

    // =========================================================================
    // PowlTapeLarge — ≤ 512 ops
    // =========================================================================

    /// A POWL v2 op tape for large programs (≤ 512 ops).
    ///
    /// Each op's predecessor/successor set is represented as a `[u64; 8]`
    /// bitmask array (512 bits total), one bit per potential peer op.
    pub struct PowlTapeLarge {
        /// Per-op predecessor bitmasks (512 bits each).
        pub pred_mask: [[u64; 8]; 512],
        /// Per-op successor bitmasks (512 bits each).
        pub succ_mask: [[u64; 8]; 512],
        /// Per-op control word (`u64::MAX` = concurrency marker).
        pub ctrl: [u64; 512],
        /// Per-op kind.
        pub op_kind: [OpKind; 512],
        /// Per-op choice-group identifier.
        pub choice_group: [u8; 512],
        /// Number of valid ops.
        pub len: u16,
        /// Interned activity label storage.
        pub label_slab: LabelSlab,
    }

    impl PowlTapeLarge {
        /// Construct an empty large tape.
        pub fn new() -> Self {
            Self {
                pred_mask: [[0u64; 8]; 512],
                succ_mask: [[0u64; 8]; 512],
                ctrl: [0u64; 512],
                op_kind: [OpKind::Silent; 512],
                choice_group: [0u8; 512],
                len: 0,
                label_slab: LabelSlab::new(),
            }
        }

        /// Returns `true` when the op at `idx` is a concurrency marker.
        #[inline(always)]
        pub fn is_concur(&self, idx: usize) -> bool {
            eq_mask_u64(self.ctrl[idx], u64::MAX) != 0
        }
    }

    impl Default for PowlTapeLarge {
        fn default() -> Self {
            Self::new()
        }
    }

    // =========================================================================
    // ConcurrencyGuardTable — compiled minimal-nonface admission gate
    // =========================================================================

    /// A compiled minimal nonface, ready to be checked against a candidate
    /// ready-set at scheduling time.
    ///
    /// `members` is expressed in **tape-slot-index space** (i.e. the same
    /// numbering as `Powl64Op` array indices / `PowlNodeId` numeric
    /// values), *not* `ActionOccurrenceId` space — the re-keying from the
    /// source `ExecutableConcurrencyComplex` (which stays
    /// `ActionOccurrenceId`-keyed all the way through
    /// [`crate::model::PowlModel::concurrency`]) into this tape-slot space
    /// happens once, at compile time, in
    /// [`crate::compiler::v2::compile_powl_v2`] — see that function's doc
    /// comment for why that is the right layer to do it in.
    ///
    /// `witness_digest` points at the full conflict witness in the source
    /// `ExecutableConcurrencyComplex::conflict_witnesses` side table (the
    /// digest itself is carried through unchanged by the re-keying step;
    /// only `members`'s *interpretation* changes).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct CompiledNonFace {
        pub members: bcinr_mfw_ir::EventSet,
        pub witness_digest: bcinr_mfw_ir::Digest,
    }

    /// A compiled concurrency-guard table: the minimal nonfaces a candidate
    /// ready-set must avoid (as a subset) to be admissible for simultaneous
    /// firing this tick.
    ///
    /// An empty table (`nonfaces.is_empty()`) admits every candidate —
    /// this is the default used when no `ExecutableConcurrencyComplex` was
    /// supplied at compile time, preserving the pre-concurrency-guard
    /// scheduler behavior exactly (see [`crate::scheduler`]).
    #[derive(Debug, Clone, Default)]
    pub struct ConcurrencyGuardTable {
        pub nonfaces: Vec<CompiledNonFace>,
    }

    impl ConcurrencyGuardTable {
        /// A guard table with no recorded nonfaces — admits everything.
        pub fn empty() -> Self {
            Self {
                nonfaces: Vec::new(),
            }
        }

        /// True iff `candidate` contains none of `self.nonfaces` as a
        /// subset — mirrors
        /// [`bcinr_mfw_ir::ExecutableConcurrencyComplex::admits`] exactly
        /// (same "structurally well-formed, not proven executable-
        /// concurrent" caveat applies — see that type's doc comment and
        /// `LAW_MINIMAL_NONFACE_REPRESENTATION`).
        ///
        /// # Complexity
        ///
        /// O(`self.nonfaces.len()`) — one [`bcinr_mfw_ir::EventSet::is_subset_of`]
        /// call per recorded nonface, each itself O(1) over `EventSet`'s
        /// fixed 8-word bitset (same cost shape as the mirrored
        /// `ExecutableConcurrencyComplex::admits`, and the same reason it
        /// matters here: this is called once per ready-set candidate inside
        /// `crate::scheduler::StableMaximalSelector`'s implementation of
        /// [`crate::scheduler::ConcurrencySelector::select`]'s inner loop,
        /// every scheduler tick — see that function's own `# Complexity`
        /// note, and `bcinr-bench/benches/mfw_hotpath_bench.rs`,
        /// which benchmarks exactly this call-volume-sensitive path).
        pub fn admits(&self, candidate: &bcinr_mfw_ir::EventSet) -> bool {
            !self
                .nonfaces
                .iter()
                .any(|nf| nf.members.is_subset_of(candidate))
        }
    }

    // =========================================================================
    // Tests
    // =========================================================================

    #[cfg(test)]
    mod tests {
        use super::*;

        // --- size / alignment ---

        #[test]
        fn powl64op_is_one_cache_line() {
            assert_eq!(
                mem::size_of::<Powl64Op>(),
                64,
                "Powl64Op must be exactly 64 bytes"
            );
            assert_eq!(
                mem::align_of::<Powl64Op>(),
                64,
                "Powl64Op must be 64-byte aligned"
            );
        }

        #[test]
        fn op_kind_is_u8() {
            assert_eq!(mem::size_of::<OpKind>(), 1);
        }

        // --- eq_mask_u64 ---

        #[test]
        fn eq_mask_equal_values() {
            assert_eq!(eq_mask_u64(0, 0), u64::MAX);
            assert_eq!(eq_mask_u64(42, 42), u64::MAX);
            assert_eq!(eq_mask_u64(u64::MAX, u64::MAX), u64::MAX);
        }

        #[test]
        fn eq_mask_unequal_values() {
            assert_eq!(eq_mask_u64(0, 1), 0);
            assert_eq!(eq_mask_u64(1, 0), 0);
            assert_eq!(eq_mask_u64(u64::MAX, 0), 0);
            assert_eq!(eq_mask_u64(0, u64::MAX), 0);
        }

        // --- LabelSlab ---

        #[test]
        fn label_slab_intern_get_roundtrip() {
            let mut slab = LabelSlab::new();
            let off = slab.intern("hello");
            assert_eq!(slab.get(off), "hello");
        }

        #[test]
        fn label_slab_multiple_labels() {
            let mut slab = LabelSlab::new();
            let a = slab.intern("alpha");
            let b = slab.intern("beta");
            let c = slab.intern("gamma");
            assert_eq!(slab.get(a), "alpha");
            assert_eq!(slab.get(b), "beta");
            assert_eq!(slab.get(c), "gamma");
        }

        #[test]
        fn label_slab_deduplication() {
            let mut slab = LabelSlab::new();
            let off1 = slab.intern("dedup");
            let off2 = slab.intern("dedup");
            assert_eq!(off1, off2, "same label must return same offset");
            assert_eq!(slab.get(off1), "dedup");
        }

        #[test]
        fn label_slab_empty_string() {
            let mut slab = LabelSlab::new();
            let off = slab.intern("");
            assert_eq!(slab.get(off), "");
        }

        #[test]
        fn label_slab_unicode() {
            let mut slab = LabelSlab::new();
            let off = slab.intern("αβγ");
            assert_eq!(slab.get(off), "αβγ");
        }

        // --- PowlTape ---

        #[test]
        fn powl_tape_push_and_ready_mask() {
            let mut tape = PowlTape::new();

            let mut op0 = Powl64Op::silent();
            op0.pred_mask = 0;
            op0.succ_mask = 1 << 1;
            assert_eq!(tape.push(op0).unwrap(), 0);

            let mut op1 = Powl64Op::silent();
            op1.pred_mask = 1 << 0;
            op1.succ_mask = 0;
            assert_eq!(tape.push(op1).unwrap(), 1);

            tape.entry_op = 0;
            tape.exit_op = 1;

            let ready = tape.ready_mask();
            assert_ne!(ready & (1 << 0), 0, "op 0 should be ready");
            assert_eq!(ready & (1 << 1), 0, "op 1 should not be ready");
        }

        #[test]
        fn powl_tape_full_rejects_65th_push() {
            let mut tape = PowlTape::new();
            for _ in 0..64 {
                assert!(tape.push(Powl64Op::silent()).is_some());
            }
            assert!(tape.push(Powl64Op::silent()).is_none());
        }

        // --- Powl64Op helpers ---

        #[test]
        fn powl64op_is_concur_when_ctrl_max() {
            let mut op = Powl64Op::silent();
            op.ctrl = u64::MAX;
            op.op_kind = OpKind::Concur;
            assert!(op.is_concur());
        }

        #[test]
        fn powl64op_not_concur_when_ctrl_zero() {
            assert!(!Powl64Op::silent().is_concur());
        }

        // --- PowlTapeLarge ---

        #[test]
        fn powl_tape_large_new_is_empty() {
            let tape = PowlTapeLarge::new();
            assert_eq!(tape.len, 0);
            assert_eq!(tape.pred_mask[0], [0u64; 8]);
            assert_eq!(tape.succ_mask[511], [0u64; 8]);
        }

        #[test]
        fn powl_tape_large_is_concur() {
            let mut tape = PowlTapeLarge::new();
            tape.ctrl[7] = u64::MAX;
            assert!(tape.is_concur(7));
            assert!(!tape.is_concur(0));
        }
    }
}

// =============================================================================
// Tests (legacy tape)
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem;

    /// Legacy tape: alloc + entry_mask smoke test.
    #[test]
    fn legacy_tape_alloc_and_entry_mask() {
        let mut tape = PowlTape::new();
        let idx = tape.alloc(OpKind::Atom).unwrap();
        assert_eq!(idx, 0);
        tape.entry_mask = 1 << idx;
        assert_eq!(tape.entry_mask, 1);
    }

    /// Legacy tape: alloc returns None when full.
    #[test]
    fn legacy_tape_full() {
        let mut tape = PowlTape::new();
        for _ in 0..64 {
            assert!(tape.alloc(OpKind::Silent).is_some());
        }
        assert!(tape.alloc(OpKind::Silent).is_none());
    }

    /// Legacy Powl64Op: branch_mask and branch_count are public fields.
    #[test]
    fn legacy_op_fields() {
        let mut op = Powl64Op::new(OpKind::XorDispatch, 0);
        op.branch_mask = 0b110;
        op.branch_count = 2;
        assert_eq!(op.branch_mask, 0b110);
        assert_eq!(op.branch_count, 2);
        assert_eq!(op.kind, OpKind::XorDispatch);
    }

    /// Legacy OpKind: all expected variants are present.
    #[test]
    fn legacy_op_kind_variants() {
        let _ = OpKind::Atom;
        let _ = OpKind::Silent;
        let _ = OpKind::XorDispatch;
        let _ = OpKind::Join;
        let _ = OpKind::LoopRedo;
    }

    /// Legacy Powl64Op size (not required to be 64 bytes — legacy layout).
    #[test]
    fn legacy_op_size() {
        // The legacy op is 32 bytes (3×u64 + 3×u8 + pad[5] = 32).
        assert_eq!(mem::size_of::<Powl64Op>(), 32);
    }
}
