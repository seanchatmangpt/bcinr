//! Safe & sound workflow-net (WF-net) type, ported and adapted from
//! `~/ggen/crates/powl2-decompose/src/net.rs` (MIT OR Apache-2.0, © Sean Chatman,
//! same copyright holder as this crate) per Kourani, Park & van der Aalst,
//! "Hierarchical Decomposition of Separable Workflow-Nets" (arXiv:2602.15739),
//! Definitions 3.1, 3.3, 3.4.
//!
//! Self-contained: string-indexed places/transitions, no external Petri-net type.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A transition label: `Some(activity)` or the silent activity `tau` (`None`).
pub type Label = Option<String>;

/// A safe & sound workflow net `N = (P, T, F)` (Def 3.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfNet {
    places: BTreeSet<String>,
    transitions: BTreeMap<String, Label>,
    pt: BTreeSet<(String, String)>,
    tp: BTreeSet<(String, String)>,
    source: String,
    sink: String,
}

/// Why a candidate byte-string is not a structurally valid WF-net.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    NotWfNet(String),
    DanglingArc(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotWfNet(m) => write!(f, "not a workflow net: {m}"),
            Self::DanglingArc(m) => write!(f, "dangling arc: {m}"),
        }
    }
}

impl std::error::Error for NetError {}

impl WfNet {
    /// Build a WF-net from places, labelled transitions, and the two arc
    /// relations, then validate the Def 3.3 invariants (unique source, unique
    /// sink, connectivity).
    pub fn new(
        places: impl IntoIterator<Item = String>,
        transitions: impl IntoIterator<Item = (String, Label)>,
        pt: impl IntoIterator<Item = (String, String)>,
        tp: impl IntoIterator<Item = (String, String)>,
        source: impl Into<String>,
        sink: impl Into<String>,
    ) -> Result<Self, NetError> {
        let net = WfNet {
            places: places.into_iter().collect(),
            transitions: transitions.into_iter().collect(),
            pt: pt.into_iter().collect(),
            tp: tp.into_iter().collect(),
            source: source.into(),
            sink: sink.into(),
        };
        net.validate()?;
        Ok(net)
    }

    fn validate(&self) -> Result<(), NetError> {
        for (p, t) in &self.pt {
            if !self.places.contains(p) {
                return Err(NetError::DanglingArc(format!("({p} -> {t}): no place {p}")));
            }
            if !self.transitions.contains_key(t) {
                return Err(NetError::DanglingArc(format!(
                    "({p} -> {t}): no transition {t}"
                )));
            }
        }
        for (t, p) in &self.tp {
            if !self.transitions.contains_key(t) {
                return Err(NetError::DanglingArc(format!(
                    "({t} -> {p}): no transition {t}"
                )));
            }
            if !self.places.contains(p) {
                return Err(NetError::DanglingArc(format!("({t} -> {p}): no place {p}")));
            }
        }
        if !self.places.contains(&self.source) {
            return Err(NetError::NotWfNet(format!(
                "declared source {} not a place",
                self.source
            )));
        }
        if !self.places.contains(&self.sink) {
            return Err(NetError::NotWfNet(format!(
                "declared sink {} not a place",
                self.sink
            )));
        }
        let no_in: BTreeSet<&String> = self
            .places
            .iter()
            .filter(|p| self.pre_place(p).is_empty())
            .collect();
        if no_in.len() != 1 || !no_in.contains(&self.source) {
            return Err(NetError::NotWfNet(format!(
                "unique-source violated: places with empty pre-set = {no_in:?}"
            )));
        }
        let no_out: BTreeSet<&String> = self
            .places
            .iter()
            .filter(|p| self.post_place(p).is_empty())
            .collect();
        if no_out.len() != 1 || !no_out.contains(&self.sink) {
            return Err(NetError::NotWfNet(format!(
                "unique-sink violated: places with empty post-set = {no_out:?}"
            )));
        }
        let fwd = self.nodes_reachable_from_source();
        let bwd = self.nodes_reaching_sink();
        for p in &self.places {
            let node = format!("p:{p}");
            if !fwd.contains(&node) || !bwd.contains(&node) {
                return Err(NetError::NotWfNet(format!(
                    "place {p} not on a source->sink path"
                )));
            }
        }
        for t in self.transitions.keys() {
            let node = format!("t:{t}");
            if !fwd.contains(&node) || !bwd.contains(&node) {
                return Err(NetError::NotWfNet(format!(
                    "transition {t} not on a source->sink path"
                )));
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn places(&self) -> &BTreeSet<String> {
        &self.places
    }

    #[must_use]
    pub fn transitions(&self) -> &BTreeMap<String, Label> {
        &self.transitions
    }

    #[must_use]
    pub fn source(&self) -> &str {
        &self.source
    }

    #[must_use]
    pub fn sink(&self) -> &str {
        &self.sink
    }

    #[must_use]
    pub fn label(&self, t: &str) -> Label {
        self.transitions.get(t).cloned().flatten()
    }

    #[must_use]
    pub fn post_place(&self, p: &str) -> BTreeSet<String> {
        self.pt
            .iter()
            .filter(|(x, _)| x == p)
            .map(|(_, t)| t.clone())
            .collect()
    }

    #[must_use]
    pub fn pre_place(&self, p: &str) -> BTreeSet<String> {
        self.tp
            .iter()
            .filter(|(_, x)| x == p)
            .map(|(t, _)| t.clone())
            .collect()
    }

    #[must_use]
    pub fn post_trans(&self, t: &str) -> BTreeSet<String> {
        self.tp
            .iter()
            .filter(|(x, _)| x == t)
            .map(|(_, p)| p.clone())
            .collect()
    }

    #[must_use]
    pub fn pre_trans(&self, t: &str) -> BTreeSet<String> {
        self.pt
            .iter()
            .filter(|(_, x)| x == t)
            .map(|(p, _)| p.clone())
            .collect()
    }

    /// Free-choice iff `pre(t1) intersect pre(t2) != empty => pre(t1) == pre(t2)` (Def 3.4).
    /// Every separable WF-net is free-choice, so a non-free-choice net is a
    /// sufficient witness of non-separability.
    #[must_use]
    pub fn is_free_choice(&self) -> bool {
        let ts: Vec<&String> = self.transitions.keys().collect();
        for i in 0..ts.len() {
            let pi = self.pre_trans(ts[i]);
            for &tj in ts.iter().skip(i + 1) {
                let pj = self.pre_trans(tj);
                if !pi.is_disjoint(&pj) && pi != pj {
                    return false;
                }
            }
        }
        true
    }

    /// `t ⤳ t'` closure (Def 3.1): transitions reachable from `t` via `F+`.
    #[must_use]
    pub fn reaches(&self, t: &str) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue: VecDeque<String> = self.trans_successors(t).into_iter().collect();
        while let Some(cur) = queue.pop_front() {
            if seen.insert(cur.clone()) {
                for nxt in self.trans_successors(&cur) {
                    if !seen.contains(&nxt) {
                        queue.push_back(nxt);
                    }
                }
            }
        }
        seen
    }

    fn trans_successors(&self, t: &str) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        for p in self.post_trans(t) {
            out.extend(self.post_place(&p));
        }
        out
    }

    /// Forward restricted reachability (Def 4.5): transitions reachable from
    /// place `p` on a path that never fires `tstop`.
    #[must_use]
    pub fn fwd_restricted(&self, p: &str, tstop: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut seen_p = BTreeSet::new();
        let mut stack = vec![p.to_string()];
        while let Some(pl) = stack.pop() {
            if !seen_p.insert(pl.clone()) {
                continue;
            }
            for t in self.post_place(&pl) {
                if t == tstop {
                    continue;
                }
                result.insert(t.clone());
                for p2 in self.post_trans(&t) {
                    if !seen_p.contains(&p2) {
                        stack.push(p2);
                    }
                }
            }
        }
        result
    }

    /// Backward restricted reachability (Def 4.6): transitions from which `p`
    /// is reachable on a path that never fires `tstop`.
    #[must_use]
    pub fn bwd_restricted(&self, p: &str, tstop: &str) -> BTreeSet<String> {
        let mut result = BTreeSet::new();
        let mut seen_p = BTreeSet::new();
        let mut stack = vec![p.to_string()];
        while let Some(pl) = stack.pop() {
            if !seen_p.insert(pl.clone()) {
                continue;
            }
            for t in self.pre_place(&pl) {
                if t == tstop {
                    continue;
                }
                result.insert(t.clone());
                for p2 in self.pre_trans(&t) {
                    if !seen_p.contains(&p2) {
                        stack.push(p2);
                    }
                }
            }
        }
        result
    }

    /// Entry places of a transition part `T'`: places that feed into `T'` and
    /// are either the net source or fed from outside `T'`.
    #[must_use]
    pub fn entry_places(&self, part: &BTreeSet<String>) -> BTreeSet<String> {
        self.places
            .iter()
            .filter(|p| {
                let post = self.post_place(p);
                !post.is_disjoint(part)
                    && (*p == &self.source || self.pre_place(p).iter().any(|t| !part.contains(t)))
            })
            .cloned()
            .collect()
    }

    /// Exit places of a part `T'`: places fed by `T'` that are either the net
    /// sink or feed outside `T'`.
    #[must_use]
    pub fn exit_places(&self, part: &BTreeSet<String>) -> BTreeSet<String> {
        self.places
            .iter()
            .filter(|p| {
                let pre = self.pre_place(p);
                !pre.is_disjoint(part)
                    && (*p == &self.sink || self.post_place(p).iter().any(|t| !part.contains(t)))
            })
            .cloned()
            .collect()
    }

    /// Place equivalence w.r.t. part `T'`: same pre-/post-transitions inside `T'`.
    #[must_use]
    pub fn equiv_wrt(&self, p: &str, q: &str, part: &BTreeSet<String>) -> bool {
        let restrict = |s: BTreeSet<String>| -> BTreeSet<String> {
            s.into_iter().filter(|t| part.contains(t)).collect()
        };
        restrict(self.pre_place(p)) == restrict(self.pre_place(q))
            && restrict(self.post_place(p)) == restrict(self.post_place(q))
    }

    fn nodes_reachable_from_source(&self) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        let start = format!("p:{}", self.source);
        queue.push_back(start.clone());
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            for n in self.tagged_successors(&node) {
                if seen.insert(n.clone()) {
                    queue.push_back(n);
                }
            }
        }
        seen
    }

    fn nodes_reaching_sink(&self) -> BTreeSet<String> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        let start = format!("p:{}", self.sink);
        queue.push_back(start.clone());
        seen.insert(start);
        while let Some(node) = queue.pop_front() {
            for n in self.tagged_predecessors(&node) {
                if seen.insert(n.clone()) {
                    queue.push_back(n);
                }
            }
        }
        seen
    }

    fn tagged_successors(&self, node: &str) -> Vec<String> {
        if let Some(p) = node.strip_prefix("p:") {
            self.post_place(p)
                .into_iter()
                .map(|t| format!("t:{t}"))
                .collect()
        } else if let Some(t) = node.strip_prefix("t:") {
            self.post_trans(t)
                .into_iter()
                .map(|p| format!("p:{p}"))
                .collect()
        } else {
            Vec::new()
        }
    }

    fn tagged_predecessors(&self, node: &str) -> Vec<String> {
        if let Some(p) = node.strip_prefix("p:") {
            self.pre_place(p)
                .into_iter()
                .map(|t| format!("t:{t}"))
                .collect()
        } else if let Some(t) = node.strip_prefix("t:") {
            self.pre_trans(t)
                .into_iter()
                .map(|p| format!("p:{p}"))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Canonical structural signature used as a cheap "no structural progress"
    /// guard in the decomposition recursion.
    #[must_use]
    pub fn signature(&self) -> (usize, usize, usize, Vec<String>) {
        let mut labels: Vec<String> = self
            .transitions
            .values()
            .map(|l| l.clone().unwrap_or_else(|| "\u{03c4}".to_string()))
            .collect();
        labels.sort();
        (
            self.places.len(),
            self.transitions.len(),
            self.pt.len() + self.tp.len(),
            labels,
        )
    }

    /// BLAKE3 hash (hex) of the net's canonical serialization.
    #[must_use]
    pub fn content_hash(&self) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(self.source.as_bytes());
        hasher.update(b"\x00");
        hasher.update(self.sink.as_bytes());
        hasher.update(b"\x00P\x00");
        for p in &self.places {
            hasher.update(p.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.update(b"\x00T\x00");
        for (t, l) in &self.transitions {
            hasher.update(t.as_bytes());
            hasher.update(b"=");
            hasher.update(l.as_deref().unwrap_or("\u{03c4}").as_bytes());
            hasher.update(b"\x00");
        }
        hasher.update(b"\x00F\x00");
        for (p, t) in &self.pt {
            hasher.update(p.as_bytes());
            hasher.update(b">");
            hasher.update(t.as_bytes());
            hasher.update(b"\x00");
        }
        for (t, p) in &self.tp {
            hasher.update(t.as_bytes());
            hasher.update(b">");
            hasher.update(p.as_bytes());
            hasher.update(b"\x00");
        }
        hasher.finalize().to_hex().to_string()
    }
}
