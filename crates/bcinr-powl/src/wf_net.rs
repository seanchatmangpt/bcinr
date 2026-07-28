//! Workflow-net (WF-net) type with a decidable soundness check, ported and adapted from
//! `~/ggen/crates/powl2-decompose/src/net.rs` (MIT OR Apache-2.0, © Sean Chatman,
//! same copyright holder as this crate) per Kourani, Park & van der Aalst,
//! "Hierarchical Decomposition of Separable Workflow-Nets" (arXiv:2602.15739),
//! Definitions 3.1, 3.3, 3.4.
//!
//! Self-contained: string-indexed places/transitions, no external Petri-net type.

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet, VecDeque};

/// Marking cap for the reachability graph. Exceeding it yields a report with
/// `truncated = true` and every soundness verdict withheld -- an unexplored
/// state space is not evidence of soundness.
pub const MAX_REACHABLE_MARKINGS: usize = 200_000;

/// A node of the net's underlying bipartite graph.
///
/// Places and transitions live in separate namespaces, so a traversal needs to
/// carry which one it is holding. This was previously done by prefixing the
/// name with `"p:"` or `"t:"` and parsing it back with `strip_prefix` -- which
/// made node kind a property of a string that any caller could get wrong, gave
/// the parse a third outcome (neither prefix) whose handler silently returned
/// no successors, and would confuse a place literally named `t:foo` with a
/// transition. The enum has exactly two cases and no parse.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Node {
    /// A place of `P`.
    Place(String),
    /// A transition of `T`.
    Transition(String),
}

/// A borrowed place name.
///
/// Places and transitions are both named by `String`, so every signature taking
/// one of each -- `fwd_restricted(place, stop_transition)` and its backward
/// twin -- accepted the arguments in either order and returned a wrong answer
/// silently when they were swapped. These two newtypes cost nothing at runtime
/// and make that call unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlaceRef<'a>(pub &'a str);

/// A borrowed transition name. See [`PlaceRef`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransRef<'a>(pub &'a str);

/// A transition label: `Some(activity)` or the silent activity `tau` (`None`).
pub type Label = Option<String>;

/// A workflow net `N = (P, T, F)` (Def 3.3). Structural validity is enforced
/// by [`WfNet::new`]; safeness and soundness are *behavioural* and are decided
/// separately by [`WfNet::check_soundness`].
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
            let node = Node::Place(p.clone());
            if !fwd.contains(&node) || !bwd.contains(&node) {
                return Err(NetError::NotWfNet(format!(
                    "place {p} not on a source->sink path"
                )));
            }
        }
        for t in self.transitions.keys() {
            let node = Node::Transition(t.clone());
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
    pub fn fwd_restricted(
        &self,
        PlaceRef(p): PlaceRef<'_>,
        TransRef(tstop): TransRef<'_>,
    ) -> BTreeSet<String> {
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
    pub fn bwd_restricted(
        &self,
        PlaceRef(p): PlaceRef<'_>,
        TransRef(tstop): TransRef<'_>,
    ) -> BTreeSet<String> {
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
    pub fn equiv_wrt(
        &self,
        PlaceRef(p): PlaceRef<'_>,
        PlaceRef(q): PlaceRef<'_>,
        part: &BTreeSet<String>,
    ) -> bool {
        let restrict = |s: BTreeSet<String>| -> BTreeSet<String> {
            s.into_iter().filter(|t| part.contains(t)).collect()
        };
        restrict(self.pre_place(p)) == restrict(self.pre_place(q))
            && restrict(self.post_place(p)) == restrict(self.post_place(q))
    }

    fn nodes_reachable_from_source(&self) -> BTreeSet<Node> {
        self.traverse(Node::Place(self.source.clone()), Self::successors)
    }

    fn nodes_reaching_sink(&self) -> BTreeSet<Node> {
        self.traverse(Node::Place(self.sink.clone()), Self::predecessors)
    }

    /// Breadth-first closure of `step` from `start`. Forward and backward
    /// reachability differ only in which neighbour function is used, so they
    /// share this.
    fn traverse(&self, start: Node, step: fn(&Self, &Node) -> Vec<Node>) -> BTreeSet<Node> {
        let mut seen = BTreeSet::new();
        let mut queue = VecDeque::new();
        seen.insert(start.clone());
        queue.push_back(start);
        while let Some(node) = queue.pop_front() {
            for n in step(self, &node) {
                if seen.insert(n.clone()) {
                    queue.push_back(n);
                }
            }
        }
        seen
    }

    fn successors(&self, node: &Node) -> Vec<Node> {
        match node {
            Node::Place(p) => self
                .post_place(p)
                .into_iter()
                .map(Node::Transition)
                .collect(),
            Node::Transition(t) => self.post_trans(t).into_iter().map(Node::Place).collect(),
        }
    }

    fn predecessors(&self, node: &Node) -> Vec<Node> {
        match node {
            Node::Place(p) => self
                .pre_place(p)
                .into_iter()
                .map(Node::Transition)
                .collect(),
            Node::Transition(t) => self.pre_trans(t).into_iter().map(Node::Place).collect(),
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

/// A marking: token count per place, indexed by the place's position in
/// `WfNet::places()` (a `BTreeSet`, so the order is the deterministic
/// lexicographic one).
///
/// `Vec<u32>` rather than a set of place names: a set cannot represent two
/// tokens in one place, which is exactly the state an unsafe net reaches. A
/// set-valued marking silently decrements-to-zero and reports a language the
/// net does not have -- see the note on `wf_net_language` in `language.rs`.
pub type Marking = Vec<u32>;

/// The outcome of replaying a WF-net's token game to exhaustion.
///
/// Every field is a decided fact about the *explored* state space. When
/// `truncated` is set the exploration hit [`MAX_REACHABLE_MARKINGS`] and the
/// three soundness clauses are `None`, because "no counterexample found in the
/// part we looked at" is not the same claim as "no counterexample exists".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoundnessReport {
    /// Number of distinct markings reached from `[source: 1]`.
    pub reachable_markings: usize,
    /// The exploration hit [`MAX_REACHABLE_MARKINGS`] and stopped early.
    pub truncated: bool,
    /// No reachable marking puts more than one token in any place (1-safe).
    pub is_safe: bool,
    /// The first marking found carrying `> 1` token in some place, if any.
    /// Kept as evidence so a refusal can name its witness.
    pub unsafe_witness: Option<Marking>,
    /// Def 3.4 free-choice, a purely structural property (so never `None`).
    pub is_free_choice: bool,
    /// Every transition is enabled in at least one reachable marking.
    /// `None` when truncated.
    pub no_dead_transitions: Option<bool>,
    /// van der Aalst clause 1: from every reachable marking, the final marking
    /// `[sink: 1]` is reachable. `None` when truncated.
    pub option_to_complete: Option<bool>,
    /// van der Aalst clause 2: no reachable marking marks the sink while
    /// differing from the final marking (no tokens left behind). `None` when
    /// truncated.
    pub proper_completion: Option<bool>,
}

impl SoundnessReport {
    /// All three van der Aalst clauses decided and holding. A truncated report
    /// is never sound.
    #[must_use]
    pub fn is_sound(&self) -> bool {
        self.option_to_complete == Some(true)
            && self.proper_completion == Some(true)
            && self.no_dead_transitions == Some(true)
    }

    /// Sound *and* 1-safe: the precondition Algorithm 3 is proved against.
    #[must_use]
    pub fn is_safe_and_sound(&self) -> bool {
        self.is_safe && self.is_sound()
    }
}

impl WfNet {
    /// Decide safeness and the three soundness clauses by exhaustive
    /// breadth-first replay of the token game from `[source: 1]`.
    ///
    /// Complexity is the size of the reachability graph, which is exponential
    /// in the number of places in general -- hence the
    /// [`MAX_REACHABLE_MARKINGS`] cap and the `truncated` flag rather than an
    /// unbounded search.
    #[must_use]
    pub fn check_soundness(&self) -> SoundnessReport {
        let places: Vec<&String> = self.places.iter().collect();
        let index: HashMap<&str, usize> = places
            .iter()
            .enumerate()
            .map(|(i, p)| (p.as_str(), i))
            .collect();
        let n = places.len();

        // Pre-resolve each transition's pre-/post-sets to place indices once,
        // so the inner replay loop is index arithmetic only.
        let fired: Vec<(usize, Vec<usize>, Vec<usize>)> = self
            .transitions
            .keys()
            .enumerate()
            .map(|(ti, t)| {
                let pre = self
                    .pre_trans(t)
                    .iter()
                    .map(|p| index[p.as_str()])
                    .collect();
                let post = self
                    .post_trans(t)
                    .iter()
                    .map(|p| index[p.as_str()])
                    .collect();
                (ti, pre, post)
            })
            .collect();

        let mut initial = vec![0u32; n];
        initial[index[self.source.as_str()]] = 1;
        let mut final_marking = vec![0u32; n];
        final_marking[index[self.sink.as_str()]] = 1;

        // Reachability graph: markings as nodes, plus the forward edges needed
        // for the reverse pass that decides option-to-complete.
        let mut ids: HashMap<Marking, usize> = HashMap::new();
        let mut markings: Vec<Marking> = Vec::new();
        let mut succ: Vec<Vec<usize>> = Vec::new();
        let mut enabled_somewhere: HashSet<usize> = HashSet::new();
        let mut is_safe = true;
        let mut unsafe_witness = None;
        let mut truncated = false;

        ids.insert(initial.clone(), 0);
        markings.push(initial);
        succ.push(Vec::new());

        let mut queue = VecDeque::from([0usize]);
        while let Some(cur) = queue.pop_front() {
            for (ti, pre, post) in &fired {
                if !pre.iter().all(|&p| markings[cur][p] >= 1) {
                    continue;
                }
                enabled_somewhere.insert(*ti);
                let mut next = markings[cur].clone();
                for &p in pre {
                    next[p] -= 1;
                }
                for &p in post {
                    next[p] += 1;
                }
                if next.iter().any(|&c| c > 1) && unsafe_witness.is_none() {
                    is_safe = false;
                    unsafe_witness = Some(next.clone());
                }
                let id = match ids.get(&next) {
                    Some(&id) => id,
                    None => {
                        if markings.len() >= MAX_REACHABLE_MARKINGS {
                            truncated = true;
                            continue;
                        }
                        let id = markings.len();
                        ids.insert(next.clone(), id);
                        markings.push(next);
                        succ.push(Vec::new());
                        queue.push_back(id);
                        id
                    }
                };
                succ[cur].push(id);
            }
        }

        let mut report = SoundnessReport {
            reachable_markings: markings.len(),
            truncated,
            is_safe,
            unsafe_witness,
            is_free_choice: self.is_free_choice(),
            no_dead_transitions: None,
            option_to_complete: None,
            proper_completion: None,
        };
        if truncated {
            return report;
        }

        report.no_dead_transitions = Some(enabled_somewhere.len() == self.transitions.len());

        // Clause 1 -- option to complete. Reverse BFS from the final marking
        // over the graph just built: a marking can complete iff it reaches the
        // final marking, so the set that can is exactly the set the final
        // marking reaches *backwards*.
        report.option_to_complete = Some(match ids.get(&final_marking) {
            None => false,
            Some(&fin) => {
                let mut pred: Vec<Vec<usize>> = vec![Vec::new(); markings.len()];
                for (from, tos) in succ.iter().enumerate() {
                    for &to in tos {
                        pred[to].push(from);
                    }
                }
                let mut can_complete = vec![false; markings.len()];
                can_complete[fin] = true;
                let mut q = VecDeque::from([fin]);
                while let Some(m) = q.pop_front() {
                    for &p in &pred[m] {
                        if !can_complete[p] {
                            can_complete[p] = true;
                            q.push_back(p);
                        }
                    }
                }
                can_complete.iter().all(|&b| b)
            }
        });

        // Clause 2 -- proper completion. The sink being marked at all in a
        // marking that is not exactly the final one means tokens were left
        // behind (or the sink was re-marked).
        let sink_idx = index[self.sink.as_str()];
        report.proper_completion = Some(
            markings
                .iter()
                .all(|m| m[sink_idx] == 0 || *m == final_marking),
        );

        report
    }
}
