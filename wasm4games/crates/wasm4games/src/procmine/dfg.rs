//! Directly-follows graph (DFG) discovery and model comparison.
//!
//! The directly-follows relation `a → b` holds when activity `b` occurs immediately after
//! `a` in some trace. Discovering it from a log and comparing it to the model's declared
//! relation is the simplest, most robust form of process discovery / model-to-model
//! conformance in van der Aalst's toolkit.
//!
//! This is a bounded, `no_std` representation: edges are stored as a fixed-capacity,
//! de-duplicated list (a single 8-step trace contributes at most 7 edges), so discovery and
//! comparison are allocation-free.

use super::model::ChainModel;

/// A directly-follows graph as a bounded, de-duplicated edge set.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Dfg {
    edges: [(u16, u16); Self::CAP],
    len: u8,
}

/// Counts of how a discovered DFG diverges from a reference (model) DFG.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct DfgDivergence {
    /// Edges present in the discovered DFG but absent from the model (unexpected follows).
    pub extra: u32,
    /// Edges present in the model but absent from the discovered DFG (missing follows).
    pub missing: u32,
}

impl Dfg {
    /// Maximum number of distinct edges this bounded DFG can hold.
    pub const CAP: usize = 64;

    /// An empty DFG.
    #[inline]
    #[must_use = "returns an empty Dfg; bind it before observing traces"]
    pub const fn empty() -> Self {
        Self {
            edges: [(0, 0); Self::CAP],
            len: 0,
        }
    }

    /// Number of distinct edges.
    #[inline]
    #[must_use]
    pub fn edge_count(&self) -> usize {
        self.len as usize
    }

    /// Whether the edge `a → b` is present.
    #[must_use]
    pub fn has_edge(&self, a: u16, b: u16) -> bool {
        self.edges[..self.len as usize].contains(&(a, b))
    }

    /// Record the consecutive `a → b` pairs of one `trace` (activity slice), de-duplicated.
    ///
    /// Saturates silently at [`Self::CAP`]; excess edges are dropped.
    pub fn observe(&mut self, trace: &[u16]) {
        for pair in trace.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if !self.has_edge(a, b) {
                let idx = self.len as usize;
                if idx < Self::CAP {
                    self.edges[idx] = (a, b);
                    self.len += 1;
                }
            }
        }
    }

    /// Merge another DFG into this one (set union of edges).
    pub fn merge(&mut self, other: &Dfg) {
        for &(a, b) in &other.edges[..other.len as usize] {
            if !self.has_edge(a, b) {
                let idx = self.len as usize;
                if idx < Self::CAP {
                    self.edges[idx] = (a, b);
                    self.len += 1;
                }
            }
        }
    }

    /// The directly-follows graph implied by a model's declared activity order.
    #[must_use = "returns the model's DFG; bind or compare it"]
    pub fn from_model(m: &ChainModel) -> Self {
        let mut dfg = Self::empty();
        dfg.observe(m.activities.as_slice());
        dfg
    }

    /// Count edges in `self` not in `model_dfg` (extra) and edges in `model_dfg` not in
    /// `self` (missing). A discovered DFG identical to the model has `{0, 0}`.
    #[must_use]
    pub fn order_divergence(&self, model_dfg: &Dfg) -> DfgDivergence {
        let mut extra = 0u32;
        for &(a, b) in &self.edges[..self.len as usize] {
            extra += u32::from(!model_dfg.has_edge(a, b));
        }
        let mut missing = 0u32;
        for &(a, b) in &model_dfg.edges[..model_dfg.len as usize] {
            missing += u32::from(!self.has_edge(a, b));
        }
        DfgDivergence { extra, missing }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::CHAIN_MODELS;
    use super::*;

    #[test]
    fn model_dfg_has_exactly_the_path_edges() {
        let m = &CHAIN_MODELS[2]; // combat_hit: [1,5,14,3,15,71,9,20]
        let dfg = Dfg::from_model(m);
        assert_eq!(dfg.edge_count(), 7);
        for w in m.activities.windows(2) {
            assert!(dfg.has_edge(w[0], w[1]));
        }
        assert!(!dfg.has_edge(m.activities[0], m.activities[2])); // not directly-follows
    }

    #[test]
    fn discovered_self_trace_matches_model_with_zero_divergence() {
        for m in CHAIN_MODELS {
            let model_dfg = Dfg::from_model(m);
            let mut discovered = Dfg::empty();
            discovered.observe(m.activities.as_slice());
            let d = discovered.order_divergence(&model_dfg);
            assert_eq!(
                d,
                DfgDivergence {
                    extra: 0,
                    missing: 0
                },
                "chain {}",
                m.name
            );
        }
    }

    #[test]
    fn reordered_trace_introduces_extra_edges() {
        let m = &CHAIN_MODELS[2];
        let model_dfg = Dfg::from_model(m);
        let mut trace = m.activities;
        trace.swap(0, 4);
        let mut discovered = Dfg::empty();
        discovered.observe(&trace);
        let d = discovered.order_divergence(&model_dfg);
        assert!(d.extra > 0, "a reorder must add unexpected follows edges");
    }

    #[test]
    fn merge_is_idempotent_union() {
        let m = &CHAIN_MODELS[0];
        let a = Dfg::from_model(m);
        let mut b = a;
        b.merge(&a);
        assert_eq!(a, b, "merging a DFG with itself is a no-op");
    }
}
