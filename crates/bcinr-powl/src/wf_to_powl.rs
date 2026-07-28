//! WF-net -> POWL 2.0 decomposition, ported and adapted from
//! `~/ggen/crates/powl2-decompose/src/decompose.rs` (MIT OR Apache-2.0,
//! © Sean Chatman, same copyright holder as this crate) per Kourani, Park &
//! van der Aalst, "Hierarchical Decomposition of Separable Workflow-Nets"
//! (arXiv:2602.15739), Algorithms 1-3, Defs 4.1-4.8.
//!
//! Retargeted from the source crate's own `Powl` output type to this crate's
//! `Powl2Model` (`crate::powl2::Powl2Model`) so the decomposition composes
//! directly with the existing v2 tape compiler.

use std::collections::{BTreeMap, BTreeSet};

use crate::language::{powl2_language, wf_net_language};
use crate::powl2::Powl2Model;
use crate::wf_net::{Label, NetError, WfNet};

/// Machine-readable reason a WF-net (or sub-net) was refused as non-separable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefusalReason {
    /// No base case and no valid partition at this recursion level -- the
    /// `FallThrough(N)` of Algorithm 3 line 25, treated as a conversion
    /// failure exactly as the paper's own implementation does (§4.4).
    IrreducibleFragment { depth: usize },
    /// The bounded recursion budget was exhausted before termination.
    BudgetExhausted { budget: usize },
    /// Algorithm 3 produced a structurally-valid decomposition, but its
    /// denotational language (`powl2_language`) disagrees with the WF-net's
    /// own token-game replay (`wf_net_language`) at the checked bound, so it
    /// is refused rather than returned as if it were correct.
    ///
    /// This names exactly the obligation discharged: agreement of two bounded
    /// enumerations. It is *not* the paper's Theorem 5.5 (Correctness), whose
    /// language-preservation content is carried by Lemmas 5.3 and 5.4 and
    /// quantifies over the whole language, not a prefix of it. There is no
    /// "Theorem 1" in Kourani/Park/van der Aalst; the results are Lemmas
    /// 5.1-5.4 and Theorems 5.5/5.6.
    BoundedLanguageAgreementFailed { checked_len: usize },
    /// `convert_and_verify` was called with a bound that cannot compare
    /// anything. Both enumerators return the empty set at `max_len == 0`, so
    /// they agree vacuously and the check would return `Ok` having compared
    /// zero traces. Agreement on nothing is not evidence.
    VacuousLanguageBound,
    /// A projection (`project_mg`/`project_sm` via `normalize`) produced a
    /// candidate sub-net that fails `WfNet::new`'s own well-formedness check
    /// (e.g. a dangling arc from `uniq`/`uniq_trans`'s synthetic
    /// source/sink/tau construction not landing where expected). This is an
    /// algorithm-internal inconsistency, not a property of the caller's
    /// input -- surfaced as a typed refusal instead of a panic so it cannot
    /// crash a long-lived process (e.g. the `wf_net_to_powl` MCP tool) on
    /// adversarial-but-structurally-valid input.
    InternalNetConstruction(NetError),
    /// The model could not be recomposed into a language-equivalent WF-net --
    /// e.g. a bounded `DoRedo`, which a WF-net cycle cannot express without
    /// widening the language.
    NotRecomposable(crate::recompose::RecomposeError),
}

impl std::fmt::Display for RefusalReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IrreducibleFragment { depth } => {
                write!(f, "irreducible fragment at depth {depth}")
            }
            Self::BudgetExhausted { budget } => {
                write!(f, "bounded recursion budget {budget} exhausted")
            }
            Self::VacuousLanguageBound => write!(
                f,
                "language bound of 0 compares two empty sets: agreement is vacuous"
            ),
            Self::BoundedLanguageAgreementFailed { checked_len } => write!(
                f,
                "bounded language agreement failed at bound {checked_len}: \
                 powl2_language(convert(N)) != wf_net_language(N)"
            ),
            Self::NotRecomposable(err) => {
                write!(f, "model is not language-preservingly recomposable: {err}")
            }
            Self::InternalNetConstruction(err) => {
                write!(
                    f,
                    "internal net construction failed during projection: {err}"
                )
            }
        }
    }
}

/// A receipted refusal, carrying the BLAKE3 content address of the offending
/// (sub-)net.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub reason: RefusalReason,
    pub net_hash: String,
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "REFUSED (non-separable, net blake3:{}): {}",
            self.net_hash, self.reason
        )
    }
}

impl std::error::Error for Refusal {}

/// The maximum recursion depth (the "bounded lanes").
pub const DEFAULT_DEPTH_BUDGET: usize = 64;

/// Convert a safe & sound WF-net into an equivalent POWL 2.0 model
/// (Algorithm 3), or refuse it as non-separable.
pub fn convert(net: &WfNet) -> Result<Powl2Model, Refusal> {
    convert_with_budget(net, DEFAULT_DEPTH_BUDGET)
}

/// [`convert`], additionally gated by *bounded* language agreement: the
/// resulting model's denotational language must agree with the WF-net's own
/// token-game replay, checked up to `max_len`. [`powl2_language`] and
/// [`wf_net_language`] share no code -- the former recurses only on
/// [`Powl2Model`], the latter touches only the [`WfNet`] API -- so their
/// agreement is real evidence.
///
/// # What this does NOT establish
///
/// This is not the paper's Theorem 5.5 (Correctness). That theorem quantifies
/// over the entire language; this compares two enumerations truncated at
/// `max_len`.
///
/// The two enumerators are independent in *code* but share a *failure mode*:
/// both return the empty set at `max_len == 0`, so they agree vacuously and
/// this function returns `Ok` having compared nothing. Independence of
/// implementation does not imply independence of failure direction. Callers
/// must choose `max_len` large enough that the comparison is non-empty --
/// see `wf_net_bridge.rs`, which derives it from the recomposed net's
/// transition count and argues the bound is exact for that construction.
///
/// An unbounded check exists: `mfw/pc-powl2/differential-rust` explores
/// `WfNet` against `recompose(convert(N))` as epsilon-NFAs over the complete
/// product automaton with no trace-length bound.
pub fn convert_and_verify(
    net: &WfNet,
    budget: usize,
    max_len: usize,
) -> Result<Powl2Model, Refusal> {
    if max_len == 0 {
        return Err(Refusal {
            reason: RefusalReason::VacuousLanguageBound,
            net_hash: net.content_hash(),
        });
    }
    let model = convert_with_budget(net, budget)?;
    let denotational = powl2_language(&model, max_len);
    let replayed = wf_net_language(net, max_len);
    if denotational != replayed {
        return Err(Refusal {
            reason: RefusalReason::BoundedLanguageAgreementFailed {
                checked_len: max_len,
            },
            net_hash: net.content_hash(),
        });
    }
    Ok(model)
}

/// [`convert`] with an explicit bounded-lane depth budget.
///
/// Runs Algorithm 3 on any safe & sound WF-net, exactly as published -- there
/// is deliberately no free-choice pre-check here. The paper does not gate on
/// free-choiceness (its own Figure 7a non-free-choice example is simply run
/// and left to the ordinary fall-through), and its correctness proofs
/// (Lemmas 5.1-5.4) require only safeness, soundness, and a conflict- or
/// concurrency-hiding partition -- never Def 3.4. The range is 5.1-5.4, not
/// 5.1-5.3: 5.1/5.2 are the structural guarantees for the marked-graph and
/// state-machine projections, 5.3/5.4 the matching language-preservation
/// results. Citing only through 5.3 covers the partial-order half of the
/// recursion and silently omits the choice-graph half. Refusing up front would
/// reject nets Algorithm 3 might legitimately decompose. Non-separable input
/// still refuses, just via `IrreducibleFragment` rather than a special case.
/// [`WfNet::is_free_choice`] remains available as the Def 3.4 predicate for
/// callers that want it as a diagnostic.
pub fn convert_with_budget(net: &WfNet, budget: usize) -> Result<Powl2Model, Refusal> {
    convert_rec(net, 0, budget)
}

fn convert_rec(net: &WfNet, depth: usize, budget: usize) -> Result<Powl2Model, Refusal> {
    if depth > budget {
        return Err(Refusal {
            reason: RefusalReason::BudgetExhausted { budget },
            net_hash: net.content_hash(),
        });
    }

    if let Some(leaf) = base_case(net) {
        return Ok(leaf);
    }

    let mg = partition_mg(net);
    if mg.len() > 1 && is_conflict_hiding(net, &mg) && mg_makes_progress(net, &mg) {
        let mut children = Vec::with_capacity(mg.len());
        for part in &mg {
            children.push(convert_child(net, part, project_mg, depth, budget)?);
        }
        let order = execution_order(net, &mg);
        return Ok(Powl2Model::PartialOrder {
            children,
            edges: order.into_iter().collect(),
        });
    }

    let sm = partition_sm(net);
    if sm.len() > 1 && is_concurrency_hiding(net, &sm) && sm_makes_progress(net, &sm) {
        let mut children = Vec::with_capacity(sm.len());
        for part in &sm {
            children.push(convert_child(net, part, project_sm, depth, budget)?);
        }
        let (edges, start, end) = execution_flow(net, &sm);
        return Ok(Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        });
    }

    Err(Refusal {
        reason: RefusalReason::IrreducibleFragment { depth },
        net_hash: net.content_hash(),
    })
}

/// Convert one child part, per Algorithm 3 lines 8-10 (`Project_MG`) and
/// 18-20 (`Project_SM`): **always** project the part and recurse.
///
/// Deliberately has no singleton short-circuit. Returning `Activity`/`Silent`
/// directly for a `part.len() == 1` looks equivalent -- and for a singleton
/// with no self-loop it is, since `Project` normalizes to a bare
/// `source -> t -> sink` net that [`base_case`] answers identically -- but it
/// is wrong for a singleton `{t}` carrying a self-loop place
/// (`p ∈ •t ∩ t•` that is neither an entry nor an exit place of the part).
/// There the paper's projection keeps `p`, so `|P| = 3`, the base case
/// correctly does not fire, neither partition of a one-transition net can
/// exceed size 1, and Algorithm 3 refuses -- whereas the short-circuit would
/// answer `Activity(t)` and silently drop the loop. Recursion still
/// terminates: that projected one-transition net immediately fails both
/// partition attempts and returns `IrreducibleFragment`.
fn convert_child(
    net: &WfNet,
    part: &BTreeSet<String>,
    project: fn(&WfNet, &BTreeSet<String>) -> Result<WfNet, NetError>,
    depth: usize,
    budget: usize,
) -> Result<Powl2Model, Refusal> {
    let sub = project(net, part).map_err(|err| Refusal {
        reason: RefusalReason::InternalNetConstruction(err),
        net_hash: net.content_hash(),
    })?;
    convert_rec(&sub, depth + 1, budget)
}

fn base_case(net: &WfNet) -> Option<Powl2Model> {
    if net.transitions().len() != 1 || net.places().len() != 2 {
        return None;
    }
    let t = net.transitions().keys().next()?;
    let pre = net.pre_trans(t);
    let post = net.post_trans(t);
    if pre.len() == 1 && post.len() == 1 && pre.contains(net.source()) && post.contains(net.sink())
    {
        return Some(match net.label(t) {
            Some(l) => Powl2Model::Activity(l),
            None => Powl2Model::Silent,
        });
    }
    None
}

// -- Algorithm 1: PartitionMG (conflict-hiding) ------------------------------

struct Groups {
    parent: BTreeMap<String, String>,
}

impl Groups {
    fn singletons<'a>(ts: impl IntoIterator<Item = &'a String>) -> Self {
        Groups {
            parent: ts.into_iter().map(|t| (t.clone(), t.clone())).collect(),
        }
    }
    fn find(&mut self, x: &str) -> String {
        let p = self.parent[x].clone();
        if p == x {
            return p;
        }
        let root = self.find(&p);
        self.parent.insert(x.to_string(), root.clone());
        root
    }
    fn union_all(&mut self, members: &BTreeSet<String>) {
        let mut iter = members.iter();
        let Some(first) = iter.next() else { return };
        let root = self.find(first);
        for m in iter {
            let r = self.find(m);
            self.parent.insert(r, root.clone());
        }
    }
    fn parts(&mut self) -> Vec<BTreeSet<String>> {
        let keys: Vec<String> = self.parent.keys().cloned().collect();
        let mut by_root: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for k in keys {
            let r = self.find(&k);
            by_root.entry(r).or_default().insert(k);
        }
        by_root.into_values().collect()
    }
}

fn partition_mg(net: &WfNet) -> Vec<BTreeSet<String>> {
    let mut g = Groups::singletons(net.transitions().keys());
    let reach: BTreeMap<String, BTreeSet<String>> = net
        .transitions()
        .keys()
        .map(|t| (t.clone(), net.reaches(t)))
        .collect();

    for p in net.places() {
        let branches = net.post_place(p);
        if branches.len() <= 1 {
            continue;
        }
        let mut group = BTreeSet::new();
        for t in net.transitions().keys() {
            let reached_by = branches.iter().filter(|b| reach[*b].contains(t)).count();
            if reached_by > 0 && reached_by < branches.len() {
                group.insert(t.clone());
            }
        }
        if group.len() > 1 {
            g.union_all(&group);
        }
    }

    for p in net.places() {
        let branches = net.pre_place(p);
        if branches.len() <= 1 {
            continue;
        }
        let mut group = BTreeSet::new();
        for t in net.transitions().keys() {
            let reaches_branch = branches.iter().filter(|b| reach[t].contains(*b)).count();
            if reaches_branch > 0 && reaches_branch < branches.len() {
                group.insert(t.clone());
            }
        }
        if group.len() > 1 {
            g.union_all(&group);
        }
    }

    g.parts()
}

fn is_conflict_hiding(net: &WfNet, parts: &[BTreeSet<String>]) -> bool {
    if parts.len() < 2 {
        return false;
    }
    for p in net.places() {
        let in_entry = parts
            .iter()
            .filter(|part| net.entry_places(part).contains(p))
            .count();
        if in_entry > 1 {
            return false;
        }
        let in_exit = parts
            .iter()
            .filter(|part| net.exit_places(part).contains(p))
            .count();
        if in_exit > 1 {
            return false;
        }
    }
    for part in parts {
        let entry: Vec<String> = net.entry_places(part).into_iter().collect();
        for i in 0..entry.len() {
            for j in (i + 1)..entry.len() {
                if !net.equiv_wrt(&entry[i], &entry[j], part) {
                    return false;
                }
            }
        }
        let exit: Vec<String> = net.exit_places(part).into_iter().collect();
        for i in 0..exit.len() {
            for j in (i + 1)..exit.len() {
                if !net.equiv_wrt(&exit[i], &exit[j], part) {
                    return false;
                }
            }
        }
    }
    true
}

fn execution_order(net: &WfNet, parts: &[BTreeSet<String>]) -> BTreeSet<(usize, usize)> {
    let n = parts.len();
    let exits: Vec<BTreeSet<String>> = parts.iter().map(|p| net.exit_places(p)).collect();
    let entries: Vec<BTreeSet<String>> = parts.iter().map(|p| net.entry_places(p)).collect();
    let mut rel = BTreeSet::new();
    for (i, exit) in exits.iter().enumerate() {
        for (j, entry) in entries.iter().enumerate() {
            if i != j && !exit.is_disjoint(entry) {
                rel.insert((i, j));
            }
        }
    }
    transitive_closure(&rel, n)
}

fn transitive_closure(rel: &BTreeSet<(usize, usize)>, n: usize) -> BTreeSet<(usize, usize)> {
    let mut m = vec![vec![false; n]; n];
    for &(i, j) in rel {
        m[i][j] = true;
    }
    for k in 0..n {
        // Row `k` is cloned so the read (`row_k[j]`) and the write
        // (`m[i][j]`) never alias the same `Vec` through `m`, letting the
        // inner loop iterate `row_k` directly instead of indexing `m` by `j`.
        let row_k = m[k].clone();
        for row_i in m.iter_mut() {
            if row_i[k] {
                for (j, &reachable) in row_k.iter().enumerate() {
                    if reachable {
                        row_i[j] = true;
                    }
                }
            }
        }
    }
    let mut out = BTreeSet::new();
    for (i, row) in m.iter().enumerate() {
        for (j, &reachable) in row.iter().enumerate() {
            if reachable && i != j {
                out.insert((i, j));
            }
        }
    }
    out
}

// -- Algorithm 2: PartitionSM (concurrency-hiding) ---------------------------

fn partition_sm(net: &WfNet) -> Vec<BTreeSet<String>> {
    let mut g = Groups::singletons(net.transitions().keys());

    for tsplit in net.transitions().keys() {
        let outs = net.post_trans(tsplit);
        if outs.len() <= 1 {
            continue;
        }
        let reach: BTreeMap<String, BTreeSet<String>> = outs
            .iter()
            .map(|p| (p.clone(), net.fwd_restricted(p, tsplit)))
            .collect();
        let mut threads = BTreeSet::new();
        for t in net.transitions().keys() {
            if t == tsplit {
                continue;
            }
            let in_some = reach.values().any(|r| r.contains(t));
            let out_some = reach.values().any(|r| !r.contains(t));
            if in_some && out_some {
                threads.insert(t.clone());
            }
        }
        let mut group = threads;
        group.insert(tsplit.clone());
        if group.len() > 1 {
            g.union_all(&group);
        }
    }

    for tjoin in net.transitions().keys() {
        let ins = net.pre_trans(tjoin);
        if ins.len() <= 1 {
            continue;
        }
        let reach: BTreeMap<String, BTreeSet<String>> = ins
            .iter()
            .map(|p| (p.clone(), net.bwd_restricted(p, tjoin)))
            .collect();
        let mut threads = BTreeSet::new();
        for t in net.transitions().keys() {
            if t == tjoin {
                continue;
            }
            let in_some = reach.values().any(|r| r.contains(t));
            let out_some = reach.values().any(|r| !r.contains(t));
            if in_some && out_some {
                threads.insert(t.clone());
            }
        }
        let mut group = threads;
        group.insert(tjoin.clone());
        if group.len() > 1 {
            g.union_all(&group);
        }
    }

    g.parts()
}

fn is_concurrency_hiding(net: &WfNet, parts: &[BTreeSet<String>]) -> bool {
    parts
        .iter()
        .all(|part| net.entry_places(part).len() == 1 && net.exit_places(part).len() == 1)
}

/// `flow(N,G)` (Def 4.8): a choice graph over part indices, with the
/// artificial start/end at `n`/`n+1` — matching `Powl2Model::ChoiceGraph`'s
/// own convention directly, no separate GNode wrapper needed.
///
/// Def 4.8's edge set is
/// `E = {(i,j) | (T_i▷ ∩ ▷T_j) ≠ ∅} ∪ {(▷,i) | N_source ∈ ▷T_i} ∪ {(i,□) | N_sink ∈ T_i▷}`,
/// with **no `i ≠ j` exclusion** -- unlike [`execution_order`], which feeds
/// `order⁺` (Def 4.3) and must stay irreflexive to be a strict partial order.
/// A choice-graph self-loop `(i,i)` is exactly how POWL 2.0 represents a part
/// that can repeat: Def 3.6 places no irreflexivity constraint on `E`, and
/// modelling cycles is the whole reason choice graphs replaced POWL 1.0's
/// `↺` operator. Filtering `i == j` here would silently turn a cyclic process
/// into an acyclic one.
fn execution_flow(net: &WfNet, parts: &[BTreeSet<String>]) -> (Vec<(usize, usize)>, usize, usize) {
    let n = parts.len();
    let start = n;
    let end = n + 1;
    let exits: Vec<BTreeSet<String>> = parts.iter().map(|p| net.exit_places(p)).collect();
    let entries: Vec<BTreeSet<String>> = parts.iter().map(|p| net.entry_places(p)).collect();
    let mut edges = Vec::new();
    for (i, exit) in exits.iter().enumerate() {
        for (j, entry) in entries.iter().enumerate() {
            if !exit.is_disjoint(entry) {
                edges.push((i, j));
            }
        }
    }
    for i in 0..n {
        if entries[i].contains(net.source()) {
            edges.push((start, i));
        }
        if exits[i].contains(net.sink()) {
            edges.push((i, end));
        }
    }
    (edges, start, end)
}

// -- Projections + normalization ---------------------------------------------

fn places_touching(net: &WfNet, part: &BTreeSet<String>) -> BTreeSet<String> {
    net.places()
        .iter()
        .filter(|p| !net.pre_place(p).is_disjoint(part) || !net.post_place(p).is_disjoint(part))
        .cloned()
        .collect()
}

fn project_mg(net: &WfNet, part: &BTreeSet<String>) -> Result<WfNet, NetError> {
    let entry = net.entry_places(part);
    let exit = net.exit_places(part);
    let touching = places_touching(net, part);

    let ps = fresh(net, "ps");
    let pe = fresh(net, "pe");

    let mut places: BTreeSet<String> = touching
        .iter()
        .filter(|p| !entry.contains(*p) && !exit.contains(*p))
        .cloned()
        .collect();
    places.insert(ps.clone());
    places.insert(pe.clone());

    let mut pt: BTreeSet<(String, String)> = BTreeSet::new();
    let mut tp: BTreeSet<(String, String)> = BTreeSet::new();

    for p in &places {
        for t in net.post_place(p) {
            if part.contains(&t) {
                pt.insert((p.clone(), t.clone()));
            }
        }
        for t in net.pre_place(p) {
            if part.contains(&t) {
                tp.insert((t.clone(), p.clone()));
            }
        }
    }
    for p in &entry {
        for t in net.post_place(p) {
            if part.contains(&t) {
                pt.insert((ps.clone(), t.clone()));
            }
        }
        for t in net.pre_place(p) {
            if part.contains(&t) {
                tp.insert((t.clone(), ps.clone()));
            }
        }
    }
    for p in &exit {
        for t in net.pre_place(p) {
            if part.contains(&t) {
                tp.insert((t.clone(), pe.clone()));
            }
        }
        for t in net.post_place(p) {
            if part.contains(&t) {
                pt.insert((pe.clone(), t.clone()));
            }
        }
    }

    let transitions: BTreeMap<String, Label> =
        part.iter().map(|t| (t.clone(), net.label(t))).collect();

    normalize(places, transitions, pt, tp, ps, pe)
}

fn project_sm(net: &WfNet, part: &BTreeSet<String>) -> Result<WfNet, NetError> {
    let entry = net.entry_places(part);
    let exit = net.exit_places(part);
    let places = places_touching(net, part);
    let ps = entry
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| net.source().to_string());
    let pe = exit
        .iter()
        .next()
        .cloned()
        .unwrap_or_else(|| net.sink().to_string());

    let mut pt = BTreeSet::new();
    let mut tp = BTreeSet::new();
    for p in &places {
        for t in net.post_place(p) {
            if part.contains(&t) {
                pt.insert((p.clone(), t.clone()));
            }
        }
        for t in net.pre_place(p) {
            if part.contains(&t) {
                tp.insert((t.clone(), p.clone()));
            }
        }
    }
    let transitions: BTreeMap<String, Label> =
        part.iter().map(|t| (t.clone(), net.label(t))).collect();

    normalize(places, transitions, pt, tp, ps, pe)
}

fn normalize(
    mut places: BTreeSet<String>,
    mut transitions: BTreeMap<String, Label>,
    mut pt: BTreeSet<(String, String)>,
    mut tp: BTreeSet<(String, String)>,
    ps: String,
    pe: String,
) -> Result<WfNet, NetError> {
    let ps_has_in = tp.iter().any(|(_, p)| p == &ps);
    let pe_has_out = pt.iter().any(|(p, _)| p == &pe);

    let source = if ps_has_in {
        let new_src = uniq(&places, &transitions, "src");
        let tau = uniq_trans(&transitions, "tau_in");
        places.insert(new_src.clone());
        transitions.insert(tau.clone(), None);
        pt.insert((new_src.clone(), tau.clone()));
        tp.insert((tau, ps.clone()));
        new_src
    } else {
        ps.clone()
    };

    let sink = if pe_has_out {
        let new_sink = uniq(&places, &transitions, "snk");
        let tau = uniq_trans(&transitions, "tau_out");
        places.insert(new_sink.clone());
        transitions.insert(tau.clone(), None);
        tp.insert((tau.clone(), new_sink.clone()));
        pt.insert((pe.clone(), tau));
        new_sink
    } else {
        pe.clone()
    };

    WfNet::new(places, transitions, pt, tp, source, sink)
}

fn fresh(net: &WfNet, stem: &str) -> String {
    let mut i = 0;
    loop {
        let cand = format!("__{stem}{i}");
        if !net.places().contains(&cand) && !net.transitions().contains_key(&cand) {
            return cand;
        }
        i += 1;
    }
}

fn uniq(places: &BTreeSet<String>, transitions: &BTreeMap<String, Label>, stem: &str) -> String {
    let mut i = 0;
    loop {
        let cand = format!("__{stem}{i}");
        if !places.contains(&cand) && !transitions.contains_key(&cand) {
            return cand;
        }
        i += 1;
    }
}

fn uniq_trans(transitions: &BTreeMap<String, Label>, stem: &str) -> String {
    let mut i = 0;
    loop {
        let cand = format!("__{stem}{i}");
        if !transitions.contains_key(&cand) {
            return cand;
        }
        i += 1;
    }
}

fn mg_makes_progress(net: &WfNet, parts: &[BTreeSet<String>]) -> bool {
    let sig = net.signature();
    // A failed projection is conservatively treated as "no progress": this
    // gate then blocks the mg/sm branch in `convert_rec`, which falls
    // through to `IrreducibleFragment` rather than the more specific
    // `InternalNetConstruction` `convert_child` would have reported had the
    // gate let this partition through -- an acceptable loss of diagnostic
    // detail in exchange for never panicking, since either way the caller
    // gets a typed `Refusal`, not a crash.
    parts.iter().all(|part| match project_mg(net, part) {
        Ok(sub) => sub.signature() != sig,
        Err(_) => false,
    })
}

fn sm_makes_progress(net: &WfNet, parts: &[BTreeSet<String>]) -> bool {
    let sig = net.signature();
    parts.iter().all(|part| match project_sm(net, part) {
        Ok(sub) => sub.signature() != sig,
        Err(_) => false,
    })
}
