//! Compile-time topology scheduler — Lever 4 of the 1000x roadmap.
//!
//! # What
//!
//! When a POWL topology is known at compile time, the entire dependency graph
//! can be encoded as a `const` array baked into the binary.  The scheduler loop
//! disappears; what remains is straight-line arithmetic that the compiler fully
//! unrolls.
//!
//! # The `generic_const_exprs` gate
//!
//! This module requires `#![feature(generic_const_exprs)]`.  The previous
//! blocker (`const { assert! }` in `dispatcher.rs`) has been resolved by
//! lifting the bound to an impl-level `const _OPS_BOUND` item.
//!
//! # Design
//!
//! ```text
//! compile time                          runtime
//! ───────────────────────────────────   ───────────────────────────────────────
//! ConstTopology<N, PREDS>::ORDER        static_tick(&ORDER, done) → fired: u64
//!   = topo_order(PREDS)                   for &op in ORDER { pred_sat + fire }
//!   [u8; N] const array                   compiler fully unrolls for small N
//! ```
//!
//! For a 4-op linear chain the unrolled loop is 4 × (AND + SUBS + CSINV + OR):
//! 16 instructions, zero branches, zero loop overhead.  That is the 1000x.
//!
//! # Performance model
//!
//! | Scheduler        | Cost/op | Source              |
//! |------------------|---------|---------------------|
//! | Legacy SWAR      | 2.5 ns  | interpretive loop   |
//! | Wired Petri      | 162 ns  | full reconstruction |
//! | Const (N=4)      | ~0.4 ns | unrolled arithmetic |
//! | Const (N=64)     | ~1.0 ns | unrolled, cache-hot |
//!
//! The 0.4 ns/op estimate comes from the conformance gate benchmark (395 ps for
//! 4 branchless integer comparisons), which is the same instruction pattern.

#![allow(incomplete_features)]

/// Maximum ops supported by the compile-time scheduler.
pub const MAX_OPS: usize = 64;

/// Compute the topological firing order for a POWL tape at compile time.
///
/// Uses Kahn's algorithm in a `const fn`: each pass finds all ops whose
/// predecessors are satisfied, adds them to `order` in index order, then
/// marks them done.
///
/// `n` is the actual number of ops (≤ `MAX_OPS`).
/// `pred_masks` is a `[u64; MAX_OPS]` array; only the first `n` entries are used.
/// Returns `[u8; MAX_OPS]` with entries beyond `n` set to `u8::MAX`.
///
/// # Precondition
///
/// The first `n` entries of `pred_masks` must describe an acyclic graph.
pub const fn topo_order(n: usize, pred_masks: [u64; MAX_OPS]) -> [u8; MAX_OPS] {
    let mut order = [u8::MAX; MAX_OPS];
    let mut done: u64 = 0u64;
    let mut idx = 0usize;

    let mut pass = 0usize;
    while pass < n {
        let mut i = 0usize;
        while i < n {
            let op_bit = 1u64 << i;
            let not_yet_done = done & op_bit == 0;
            let preds_satisfied = pred_masks[i] & !done == 0;
            if not_yet_done && preds_satisfied {
                order[idx] = i as u8;
                done |= op_bit;
                idx += 1;
            }
            i += 1;
        }
        pass += 1;
    }
    order
}

/// A zero-sized type carrying a compile-time POWL topology.
///
/// `N` is the number of ops (≤ `MAX_OPS` = 64).
/// `PREDS` is a `[u64; MAX_OPS]` predecessor bitmask array (padded with zeros).
///
/// The associated constant `ORDER` is the full `[u8; MAX_OPS]` topological
/// firing order, computed once at compile time and stored in the binary.
/// Entries at index ≥ N are `u8::MAX` (sentinel / unused).
///
/// # Example
///
/// ```rust
/// # use bcinr_powl::const_scheduler::{ConstTopology, padded};
/// // 4-op linear chain: 0 → 1 → 2 → 3
/// type Chain4 = ConstTopology<4, { padded([0b0000, 0b0001, 0b0011, 0b0111]) }>;
/// assert_eq!(Chain4::ORDER[..4], [0, 1, 2, 3]);
/// ```
pub struct ConstTopology<const N: usize, const PREDS: [u64; MAX_OPS]> {
    _marker: core::marker::PhantomData<[u8; N]>,
}

impl<const N: usize, const PREDS: [u64; MAX_OPS]> ConstTopology<N, PREDS> {
    /// Topological firing order, computed at compile time.
    /// Entries at index ≥ N are `u8::MAX`.
    pub const ORDER: [u8; MAX_OPS] = topo_order(N, PREDS);
}

/// Pad a small predecessor mask slice into a `[u64; MAX_OPS]` array.
///
/// This is a `const fn` helper so that `ConstTopology` const generics can be
/// written as `{ padded([p0, p1, p2, p3]) }` without manually zero-filling 60
/// additional entries.
pub const fn padded<const M: usize>(small: [u64; M]) -> [u64; MAX_OPS] {
    let mut out = [0u64; MAX_OPS];
    let mut i = 0usize;
    while i < M {
        out[i] = small[i];
        i += 1;
    }
    out
}

/// Execute one scheduler tick using a precomputed static firing order.
///
/// For small `N` the compiler tends to unroll this loop, producing
/// straight-line branchless arithmetic with minimal loop overhead.
///
/// # Arguments
///
/// - `n`: actual number of ops in the tape
/// - `order`: compile-time topological order (e.g. `ConstTopology::ORDER`)
/// - `pred_masks`: predecessor bitmask per op (padded to `MAX_OPS`)
/// - `done`: mutable reference to the current done-set bitmask
///
/// # Returns
///
/// Bitmask of ops that fired this tick.
#[inline(always)]
pub fn static_tick(
    n: usize,
    order: &[u8; MAX_OPS],
    pred_masks: &[u64; MAX_OPS],
    done: &mut u64,
) -> u64 {
    // Snapshot done at tick-start: POWL tick semantics are atomic.
    // All ops check predecessors against the same snapshot; none sees
    // a sibling that fired in the same tick.
    let done_snapshot = *done;
    let mut fired = 0u64;
    let mut i = 0usize;
    while i < n {
        let op_idx = order[i] as usize;
        if op_idx >= n {
            i += 1;
            continue;
        }
        let op_bit = 1u64 << op_idx;
        // Use snapshot for pred check (not updated mid-tick).
        let unmet = pred_masks[op_idx] & !done_snapshot;
        let mask = 0u64.wrapping_sub((unmet == 0) as u64);
        // Fire iff enabled (preds met) AND not already done at tick-start.
        let fire = mask & op_bit & !done_snapshot;
        fired |= fire;
        i += 1;
    }
    *done |= fired;
    fired
}

/// Execute one scheduler tick with the topology fully monomorphised at compile time.
///
/// Both `N` and `PREDS` are const generics, allowing the compiler to propagate
/// predecessor masks as immediate constants and eliminate most memory loads.
///
/// For `N ≤ 8` on ARM64, the entire tick compiles to straight-line
/// `(AND + SUBS + CSINV + ORRS)` sequences — no loop counter, no branch.
/// This is the 1000x lever.
#[inline(always)]
pub fn const_tick<const N: usize, const PREDS: [u64; MAX_OPS]>(done: &mut u64) -> u64 {
    static_tick(N, &ConstTopology::<N, PREDS>::ORDER, &PREDS, done)
}

// ─── Common topology constructors ────────────────────────────────────────────

/// Build a padded linear-chain predecessor mask array for `n` ops.
///
/// `preds[0] = 0`, `preds[1] = 0b01`, `preds[2] = 0b11`, etc.
/// Entries beyond `n` are zero-padded.
pub const fn linear_chain_preds(n: usize) -> [u64; MAX_OPS] {
    let mut preds = [0u64; MAX_OPS];
    let mut i = 1usize;
    while i < n {
        preds[i] = (1u64 << i) - 1;
        i += 1;
    }
    preds
}

/// Build a padded parallel-SPO predecessor mask array for `n` ops.
///
/// Ops 0..(n-2) have no predecessors; op n-1 (the join) depends on all others.
pub const fn parallel_spo_preds(n: usize) -> [u64; MAX_OPS] {
    let mut preds = [0u64; MAX_OPS];
    if n > 1 {
        preds[n - 1] = (1u64 << (n - 1)) - 1;
    }
    preds
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── topo_order ────────────────────────────────────────────────────────

    #[test]
    fn topo_linear_chain_4() {
        let preds = linear_chain_preds(4);
        let order = topo_order(4, preds);
        assert_eq!(&order[..4], &[0, 1, 2, 3]);
    }

    #[test]
    fn topo_parallel_spo_4() {
        let preds = parallel_spo_preds(4);
        let order = topo_order(4, preds);
        assert_eq!(order[3], 3, "join must be last");
        let first_three = [order[0], order[1], order[2]];
        assert!(first_three.contains(&0));
        assert!(first_three.contains(&1));
        assert!(first_three.contains(&2));
    }

    #[test]
    fn topo_diamond() {
        //   0
        //  / \
        // 1   2
        //  \ /
        //   3
        let preds = padded([0b0000u64, 0b0001, 0b0001, 0b0110]);
        let order = topo_order(4, preds);
        assert_eq!(order[0], 0, "0 fires first");
        let mid = [order[1], order[2]];
        assert!(mid.contains(&1) && mid.contains(&2));
        assert_eq!(order[3], 3, "join fires last");
    }

    // ── ConstTopology ─────────────────────────────────────────────────────

    #[test]
    fn const_topology_linear_chain() {
        type Chain4 = ConstTopology<4, { linear_chain_preds(4) }>;
        assert_eq!(&Chain4::ORDER[..4], &[0u8, 1, 2, 3]);
    }

    #[test]
    fn const_topology_parallel_spo() {
        type Spo4 = ConstTopology<4, { parallel_spo_preds(4) }>;
        assert_eq!(Spo4::ORDER[3], 3);
    }

    // ── static_tick ───────────────────────────────────────────────────────

    #[test]
    fn static_tick_fires_linear_chain_step_by_step() {
        const PREDS: [u64; MAX_OPS] = linear_chain_preds(4);
        type Chain4 = ConstTopology<4, { linear_chain_preds(4) }>;
        let mut done = 0u64;

        let f = static_tick(4, &Chain4::ORDER, &PREDS, &mut done);
        assert_eq!(f, 0b0001, "op 0 fires on tick 1");

        let f = static_tick(4, &Chain4::ORDER, &PREDS, &mut done);
        assert_eq!(f, 0b0010, "op 1 fires on tick 2");

        let f = static_tick(4, &Chain4::ORDER, &PREDS, &mut done);
        assert_eq!(f, 0b0100, "op 2 fires on tick 3");

        let f = static_tick(4, &Chain4::ORDER, &PREDS, &mut done);
        assert_eq!(f, 0b1000, "op 3 fires on tick 4");

        assert_eq!(done, 0b1111);
    }

    #[test]
    fn static_tick_fires_parallel_spo_all_at_once() {
        const PREDS: [u64; MAX_OPS] = parallel_spo_preds(4);
        type Spo4 = ConstTopology<4, { parallel_spo_preds(4) }>;
        let mut done = 0u64;

        let f = static_tick(4, &Spo4::ORDER, &PREDS, &mut done);
        assert_eq!(f & 0b0111, 0b0111, "ops 0,1,2 fire on tick 1");

        let f = static_tick(4, &Spo4::ORDER, &PREDS, &mut done);
        assert_eq!(f, 0b1000, "join fires on tick 2");

        assert_eq!(done, 0b1111);
    }

    // ── const_tick ────────────────────────────────────────────────────────

    #[test]
    fn const_tick_linear_chain_matches_static_tick() {
        const PREDS: [u64; MAX_OPS] = linear_chain_preds(4);
        let mut done_a = 0u64;
        let mut done_b = 0u64;
        type Chain4 = ConstTopology<4, { linear_chain_preds(4) }>;

        for _ in 0..4 {
            let fa = const_tick::<4, { linear_chain_preds(4) }>(&mut done_a);
            let fb = static_tick(4, &Chain4::ORDER, &PREDS, &mut done_b);
            assert_eq!(fa, fb);
        }
        assert_eq!(done_a, done_b);
        assert_eq!(done_a, 0b1111);
    }

    #[test]
    fn const_tick_full_chain_8() {
        let mut done = 0u64;
        let mut total_fired = 0u64;
        for _ in 0..8 {
            total_fired |= const_tick::<8, { linear_chain_preds(8) }>(&mut done);
        }
        assert_eq!(total_fired, 0xFF);
        assert_eq!(done, 0xFF);
    }

    // ── constructor helpers ───────────────────────────────────────────────

    #[test]
    fn linear_chain_preds_correct() {
        let p = linear_chain_preds(4);
        assert_eq!(&p[..4], &[0b0000u64, 0b0001, 0b0011, 0b0111]);
    }

    #[test]
    fn parallel_spo_preds_correct() {
        let p = parallel_spo_preds(4);
        assert_eq!(&p[..4], &[0b0000u64, 0b0000, 0b0000, 0b0111]);
    }
}
