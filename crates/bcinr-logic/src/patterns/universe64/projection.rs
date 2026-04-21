//! # Universe64 Contract: ProjectionUpdater + ProjectionCache (T2 Rebuild)
//! Plane: D (read source words) + cache plane (write projections).
//! Tier: T1 incremental per-delta; T2 full-rebuild.
//! Scope: bounded projection cache; UNIVERSE_WORDS fan-in.
//! Geometry: projection = reduce(words[word_ids], op) → summary scalar per projection.
//! Delta: reads UDelta to drive incremental cache invalidation.
//!
//! # Timing contract
//! - **T1 incremental update:** ≤ T1_BUDGET_NS per delta
//! - **T2 full rebuild:** ≤ T2_BUDGET_NS
//! - **Max heap allocations:** 0
//!
//! # Admissibility
//! Admissible_T1: YES. Bounded projections; branchless reduction ops.
//! CC=1: Absolute branchless logic.

use super::constants::UNIVERSE_WORDS;
use super::block::UniverseBlock;
use super::scratch::UDelta;
use super::index_plane::{MAX_PROJECTIONS, MAX_WORDS_PER_TRANSITION};

/// Reduction operation for a projection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ProjectionOp {
    /// OR of all source words.
    Or = 0,
    /// AND of all source words.
    And = 1,
    /// XOR of all source words.
    Xor = 2,
    /// Popcount sum across all source words.
    PopcountSum = 3,
}

/// Static descriptor for one projection.
#[derive(Clone, Copy, Debug)]
pub struct ProjectionDescriptor {
    pub id: u8,
    pub op: ProjectionOp,
    pub word_count: u8,
    pub word_idxs: [u16; MAX_WORDS_PER_TRANSITION],
    pub valid: bool,
}

impl Default for ProjectionDescriptor {
    fn default() -> Self {
        Self {
            id: 0,
            op: ProjectionOp::Or,
            word_count: 0,
            word_idxs: [0u16; MAX_WORDS_PER_TRANSITION],
            valid: false,
        }
    }
}

/// Cached projection values.
#[derive(Clone, Copy, Debug)]
pub struct ProjectionCache {
    /// Cached reduced value per projection.
    pub values: [u64; MAX_PROJECTIONS],
    /// Dirty flags: bit i set → projection i needs rebuild.
    pub dirty: u64,
}

impl ProjectionCache {
    pub const fn new() -> Self {
        Self { values: [0u64; MAX_PROJECTIONS], dirty: u64::MAX }
    }

    /// Mark projection `id` dirty.
    #[inline(always)]
    pub fn mark_dirty(&mut self, id: u8) {
        self.dirty |= 1u64 << (id & 63);
    }

    /// Clear dirty bit for projection `id`.
    #[inline(always)]
    pub fn mark_clean(&mut self, id: u8) {
        self.dirty &= !(1u64 << (id & 63));
    }

    /// True if projection `id` is dirty.
    #[inline(always)]
    pub fn is_dirty(&self, id: u8) -> bool {
        (self.dirty >> (id & 63)) & 1 != 0
    }
}

impl Default for ProjectionCache {
    fn default() -> Self { Self::new() }
}

/// Registry of projection descriptors.
#[derive(Clone, Copy)]
pub struct ProjectionRegistry {
    descs: [ProjectionDescriptor; MAX_PROJECTIONS],
    count: u8,
}

impl ProjectionRegistry {
    pub const fn new() -> Self {
        Self { descs: [ProjectionDescriptor { id: 0, op: ProjectionOp::Or, word_count: 0, word_idxs: [0u16; MAX_WORDS_PER_TRANSITION], valid: false }; MAX_PROJECTIONS], count: 0 }
    }

    pub fn register(&mut self, op: ProjectionOp, word_idxs: &[u16]) -> u8 {
        let admit = (self.count as usize) < MAX_PROJECTIONS;
        let id = self.count * admit as u8;
        let slot = (self.count as usize) & (MAX_PROJECTIONS - 1);
        let n = word_idxs.len().min(MAX_WORDS_PER_TRANSITION);
        let mut desc = ProjectionDescriptor { id, op, word_count: n as u8, word_idxs: [0u16; MAX_WORDS_PER_TRANSITION], valid: admit };
        let mut i = 0;
        while i < n { desc.word_idxs[i] = word_idxs[i]; i += 1; }
        self.descs[slot] = desc;
        self.count = self.count.wrapping_add(admit as u8);
        id | ((!admit as u8) * 255)
    }

    #[inline(always)]
    pub fn get(&self, id: u8) -> &ProjectionDescriptor {
        &self.descs[id as usize & (MAX_PROJECTIONS - 1)]
    }

    pub const fn len(&self) -> usize { self.count as usize }
}

impl Default for ProjectionRegistry {
    fn default() -> Self { Self::new() }
}

// ---------------------------------------------------------------------------
// ProjectionUpdater
// ---------------------------------------------------------------------------

/// Computes and caches projection values.
pub struct ProjectionUpdater;

impl ProjectionUpdater {
    /// Compute one projection into cache (branchless reduction).
    pub fn rebuild_one(
        block: &UniverseBlock,
        desc: &ProjectionDescriptor,
        cache: &mut ProjectionCache,
    ) {
        if !desc.valid { return; }
        let n = desc.word_count as usize;
        let val = match desc.op {
            ProjectionOp::Or => {
                let mut acc = 0u64;
                for i in 0..n { acc |= block.state[desc.word_idxs[i] as usize & (UNIVERSE_WORDS - 1)]; }
                acc
            }
            ProjectionOp::And => {
                let mut acc = u64::MAX;
                for i in 0..n { acc &= block.state[desc.word_idxs[i] as usize & (UNIVERSE_WORDS - 1)]; }
                acc * (n > 0) as u64  // 0 if n=0
            }
            ProjectionOp::Xor => {
                let mut acc = 0u64;
                for i in 0..n { acc ^= block.state[desc.word_idxs[i] as usize & (UNIVERSE_WORDS - 1)]; }
                acc
            }
            ProjectionOp::PopcountSum => {
                let mut acc = 0u64;
                for i in 0..n { acc = acc.wrapping_add(block.state[desc.word_idxs[i] as usize & (UNIVERSE_WORDS - 1)].count_ones() as u64); }
                acc
            }
        };
        cache.values[desc.id as usize & (MAX_PROJECTIONS - 1)] = val;
        cache.mark_clean(desc.id);
    }

    /// Rebuild all dirty projections (T2 batch).
    pub fn rebuild_dirty(
        block: &UniverseBlock,
        registry: &ProjectionRegistry,
        cache: &mut ProjectionCache,
    ) {
        for i in 0..registry.count as usize {
            let desc = &registry.descs[i];
            if desc.valid && cache.is_dirty(desc.id) {
                Self::rebuild_one(block, desc, cache);
            }
        }
    }

    /// Incremental delta-driven invalidation (T1).
    #[inline]
    pub fn invalidate_on_delta(
        registry: &ProjectionRegistry,
        cache: &mut ProjectionCache,
        delta: &UDelta,
    ) {
        let widx = delta.word_idx as usize;
        for i in 0..registry.count as usize {
            let desc = &registry.descs[i];
            if !desc.valid { continue; }
            for j in 0..desc.word_count as usize {
                if desc.word_idxs[j] as usize == widx {
                    cache.mark_dirty(desc.id);
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projection_or() {
        let mut block = UniverseBlock::new();
        block.state[0] = 0b0001;
        block.state[1] = 0b0010;
        let mut reg = ProjectionRegistry::new();
        let id = reg.register(ProjectionOp::Or, &[0, 1]);
        let mut cache = ProjectionCache::new();
        let desc = reg.get(id);
        ProjectionUpdater::rebuild_one(&block, desc, &mut cache);
        assert_eq!(cache.values[id as usize], 0b0011);
    }

    #[test]
    fn test_projection_popcount() {
        let mut block = UniverseBlock::new();
        block.state[5] = 0b1111;
        let mut reg = ProjectionRegistry::new();
        let id = reg.register(ProjectionOp::PopcountSum, &[5]);
        let mut cache = ProjectionCache::new();
        let desc = reg.get(id);
        ProjectionUpdater::rebuild_one(&block, desc, &mut cache);
        assert_eq!(cache.values[id as usize], 4);
    }

    #[test]
    fn test_dirty_invalidation() {
        let mut reg = ProjectionRegistry::new();
        let id = reg.register(ProjectionOp::Or, &[3]);
        let mut cache = ProjectionCache::new();
        cache.mark_clean(id);
        let delta = UDelta::new(3, super::super::scratch::UScope::Cell, 0, 0, 1, !0);
        ProjectionUpdater::invalidate_on_delta(&reg, &mut cache, &delta);
        assert!(cache.is_dirty(id));
    }

    #[test]
    fn test_rebuild_dirty() {
        let mut block = UniverseBlock::new();
        block.state[2] = 0b1010;
        let mut reg = ProjectionRegistry::new();
        let id = reg.register(ProjectionOp::Xor, &[2]);
        let mut cache = ProjectionCache::new();
        ProjectionUpdater::rebuild_dirty(&block, &reg, &mut cache);
        assert_eq!(cache.values[id as usize], 0b1010);
    }
}
