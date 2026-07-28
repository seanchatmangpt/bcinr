//! L(psi) bounded language enumerator (Def 3.8/3.9), ported and adapted from
//! `~/ggen/crates/powl2-decompose/src/{powl.rs,language.rs}` (MIT OR
//! Apache-2.0, © Sean Chatman, same copyright holder as this crate).
//!
//! Two independently-computed languages, kept as separate functions on
//! purpose: [`powl2_language`] is denotational (computed from the
//! `Powl2Model` definition itself) and [`wf_net_language`] is computed by
//! exhaustive WF-net token-game replay. Their agreement on a converted model
//! is the genuine differential check for Algorithm 3's correctness.

use std::collections::{BTreeMap, BTreeSet};

use crate::powl2::{validate_edges, Powl2Error, Powl2Model};
use crate::wf_net::{Marking, WfNet};

/// A bounded language: all label sequences of length <= `max_len`.
pub type Language = BTreeSet<Vec<String>>;

/// `L(psi)` (Def 3.9), truncated to traces of length <= `max_len`. For
/// acyclic models with `max_len` at least the longest trace this is the exact
/// language.
///
/// # Errors
///
/// Returns [`Powl2Error::InvalidEdge`] if a `PartialOrder` edge names a child
/// index outside `0..children.len()`. Such an edge used to be *dropped*, which
/// made the enumerated language a strict superset of the model's: the missing
/// precedence constraint admitted interleavings the model forbids. That
/// matters because this function is one half of the differential check in
/// `wf_to_powl::convert_and_verify` -- a too-permissive left-hand side can
/// make a divergent conversion compare equal, i.e. the verifier passes
/// *because* it was weakened. A malformed edge is now refused, not silently
/// reinterpreted.
pub fn powl2_language(model: &Powl2Model, max_len: usize) -> Result<Language, Powl2Error> {
    Ok(match model {
        Powl2Model::Activity(label) => {
            let mut l = Language::new();
            if max_len >= 1 {
                l.insert(vec![label.clone()]);
            }
            l
        }
        Powl2Model::Silent => {
            let mut l = Language::new();
            l.insert(Vec::new());
            l
        }
        Powl2Model::Sequence(children) => sequence_language(children, max_len)?,
        Powl2Model::PartialOrder { children, edges } => {
            let child_langs: Vec<Language> = children
                .iter()
                .map(|c| powl2_language(c, max_len))
                .collect::<Result<_, _>>()?;
            let n = children.len();
            // Reuses `powl2::validate_edges` -- the same bound check
            // `validate_powl2` applies to a `PartialOrder` -- rather than
            // inventing a second notion of a well-formed edge.
            validate_edges(n, edges)?;
            let mut prec = vec![vec![false; n]; n];
            for &(i, j) in edges {
                prec[i][j] = true;
            }
            order_preserving_shuffle(&child_langs, &prec, max_len)
        }
        Powl2Model::ChoiceGraph {
            children,
            edges,
            start,
            end,
        } => {
            let child_langs: Vec<Language> = children
                .iter()
                .map(|c| powl2_language(c, max_len))
                .collect::<Result<_, _>>()?;
            choice_graph_language(&child_langs, edges, *start, *end, children.len(), max_len)
        }
        Powl2Model::DoRedo {
            body,
            redo,
            max_redos,
        } => do_redo_language(body, redo, *max_redos, max_len)?,
    })
}

fn sequence_language(children: &[Powl2Model], max_len: usize) -> Result<Language, Powl2Error> {
    let mut current: Language = Language::new();
    current.insert(Vec::new());
    for child in children {
        let child_lang = powl2_language(child, max_len)?;
        let mut next = Language::new();
        for prefix in &current {
            for suffix in &child_lang {
                if prefix.len() + suffix.len() > max_len {
                    continue;
                }
                let mut combined = prefix.clone();
                combined.extend(suffix.iter().cloned());
                next.insert(combined);
            }
        }
        current = next;
    }
    Ok(current)
}

/// Order-preserving shuffle (Def 3.8): for every choice of one sequence per
/// child, all interleavings respecting the strict partial order `prec`.
fn order_preserving_shuffle(
    child_langs: &[Language],
    prec: &[Vec<bool>],
    max_len: usize,
) -> Language {
    let n = child_langs.len();
    if n == 0 {
        let mut s = Language::new();
        s.insert(Vec::new());
        return s;
    }
    let mut result = Language::new();
    let selections = cartesian(child_langs);
    for combo in selections {
        interleave(&combo, prec, n, max_len, &mut result);
    }
    result
}

fn cartesian(child_langs: &[Language]) -> Vec<Vec<Vec<String>>> {
    let mut acc: Vec<Vec<Vec<String>>> = vec![Vec::new()];
    for lang in child_langs {
        let mut next = Vec::new();
        for prefix in &acc {
            for seq in lang {
                let mut p = prefix.clone();
                p.push(seq.clone());
                next.push(p);
            }
        }
        acc = next;
    }
    acc
}

fn interleave(
    combo: &[Vec<String>],
    prec: &[Vec<bool>],
    n: usize,
    max_len: usize,
    result: &mut Language,
) {
    let progress: Vec<usize> = vec![0; n];
    let mut current: Vec<String> = Vec::new();
    interleave_rec(combo, prec, n, &progress, &mut current, max_len, result);
}

#[allow(clippy::too_many_arguments)]
fn interleave_rec(
    combo: &[Vec<String>],
    prec: &[Vec<bool>],
    n: usize,
    progress: &[usize],
    current: &mut Vec<String>,
    max_len: usize,
    result: &mut Language,
) {
    if current.len() > max_len {
        return;
    }
    if (0..n).all(|i| progress[i] >= combo[i].len()) {
        result.insert(current.clone());
        return;
    }
    for i in 0..n {
        if progress[i] >= combo[i].len() {
            continue;
        }
        let blocked = (0..n).any(|j| prec[j][i] && progress[j] < combo[j].len());
        if blocked {
            continue;
        }
        let tok = combo[i][progress[i]].clone();
        current.push(tok);
        let mut next = progress.to_vec();
        next[i] += 1;
        interleave_rec(combo, prec, n, &next, current, max_len, result);
        current.pop();
    }
}

/// Bound on how many consecutive *zero-progress* re-entries a choice-graph
/// node may have during language enumeration. See [`NodeBudget`]: this
/// bounds stalls (revisits producing no trace growth, e.g. hops through an
/// epsilon/`Silent` child), not raw revisits -- a node revisited only after
/// genuine trace progress elsewhere on the path is never charged against
/// this limit, which is what makes cyclic graphs with a silent branch
/// enumerate completely up to `max_len` rather than being pruned by
/// zero-length hops eating the same budget as real progress.
const MAX_CYCLE_UNROLL: usize = 2;

/// Per-node cycle-unrolling bookkeeping for [`cg_dfs`]: the prefix length
/// recorded at this node's most recent entry on the current DFS path, and
/// how many consecutive re-entries since then produced no growth.
#[derive(Clone, Copy)]
struct NodeBudget {
    /// `prefix.len()` at the last visit to this node on the current path,
    /// or `None` before the first visit.
    last_progress: Option<usize>,
    /// Consecutive re-entries with `prefix.len()` no greater than
    /// `last_progress` (i.e. no growth since the previous visit).
    stall: usize,
}

/// Choice-graph language (Def 3.9): union over every start->end path of the
/// concatenation of the visited children's languages, bounded by `max_len`
/// and by a per-node stall cap for cycles.
fn choice_graph_language(
    child_langs: &[Language],
    edges: &[(usize, usize)],
    start: usize,
    end: usize,
    n_children: usize,
    max_len: usize,
) -> Language {
    let max_node = edges
        .iter()
        .flat_map(|&(a, b)| [a, b])
        .max()
        .unwrap_or(end)
        .max(end)
        .max(start);
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); max_node + 1];
    for &(a, b) in edges {
        adj[a].push(b);
    }
    let mut result = Language::new();
    let mut prefix: Vec<String> = Vec::new();
    let mut budgets = vec![
        NodeBudget {
            last_progress: None,
            stall: 0
        };
        max_node + 1
    ];
    cg_dfs(
        &adj,
        child_langs,
        start,
        end,
        n_children,
        max_len,
        &mut prefix,
        &mut budgets,
        &mut result,
    );
    result
}

/// DFS over the choice graph, accumulating `prefix` and pruning via
/// `budgets` (see [`NodeBudget`]/[`MAX_CYCLE_UNROLL`]). Termination: split
/// any root-to-current path into maximal runs during which `prefix.len()`
/// stays constant ("stalls"). Within one stall, no node's `last_progress`
/// changes, so each re-entry to the same node increments its `stall`
/// counter exactly like a raw-revisit counter would, and is pruned once
/// `stall > MAX_CYCLE_UNROLL` -- bounding a stall's length by
/// `(max_node + 1) * (MAX_CYCLE_UNROLL + 1)` calls. Since `prefix.len()` is
/// non-decreasing along a path and capped at `max_len` (the `cur_len >
/// max_len` guard), there are at most `max_len + 1` such stalls before the
/// path must end. Both factors are finite, so recursion depth is bounded by
/// their product regardless of graph cycles.
#[allow(clippy::too_many_arguments)]
fn cg_dfs(
    adj: &[Vec<usize>],
    child_langs: &[Language],
    node: usize,
    end: usize,
    n_children: usize,
    max_len: usize,
    prefix: &mut Vec<String>,
    budgets: &mut [NodeBudget],
    result: &mut Language,
) {
    if node == end {
        result.insert(prefix.clone());
        return;
    }
    let cur_len = prefix.len();
    if cur_len > max_len {
        return;
    }
    let old = budgets[node];
    let progressed = match old.last_progress {
        Some(last) => cur_len > last,
        None => true,
    };
    let new_stall = if progressed { 0 } else { old.stall + 1 };
    if new_stall > MAX_CYCLE_UNROLL {
        return;
    }
    budgets[node] = NodeBudget {
        last_progress: Some(cur_len),
        stall: new_stall,
    };
    if node < n_children {
        for seq in &child_langs[node] {
            if prefix.len() + seq.len() > max_len {
                continue;
            }
            let added = seq.len();
            prefix.extend(seq.iter().cloned());
            for &nxt in &adj[node] {
                cg_dfs(
                    adj,
                    child_langs,
                    nxt,
                    end,
                    n_children,
                    max_len,
                    prefix,
                    budgets,
                    result,
                );
            }
            let new_len = prefix.len() - added;
            prefix.truncate(new_len);
        }
    } else {
        for &nxt in &adj[node] {
            cg_dfs(
                adj,
                child_langs,
                nxt,
                end,
                n_children,
                max_len,
                prefix,
                budgets,
                result,
            );
        }
    }
    budgets[node] = old;
}

/// `DoRedo` language: `body`, then zero-or-more (up to `max_redos`) further
/// `redo -> body` cycles, keeping every intermediate iteration count (not
/// just the maximum).
fn do_redo_language(
    body: &Powl2Model,
    redo: &Powl2Model,
    max_redos: u8,
    max_len: usize,
) -> Result<Language, Powl2Error> {
    let body_lang = powl2_language(body, max_len)?;
    let redo_lang = powl2_language(redo, max_len)?;

    let mut result: Language = Language::new();
    result.extend(body_lang.iter().cloned());

    let mut current: Language = body_lang.clone();
    for _ in 0..max_redos {
        let mut next: Language = Language::new();
        for t_b in &current {
            for t_r in &redo_lang {
                for t_body in &body_lang {
                    if t_b.len() + t_r.len() + t_body.len() > max_len {
                        continue;
                    }
                    let mut new_trace = t_b.clone();
                    new_trace.extend(t_r.iter().cloned());
                    new_trace.extend(t_body.iter().cloned());
                    next.insert(new_trace);
                }
            }
        }
        if next.is_empty() {
            break;
        }
        result.extend(next.iter().cloned());
        current = next;
    }
    Ok(result)
}

/// `L(N)`: the bounded language of `net`, computed by exhaustive replay of
/// firing sequences from `[source]` to `[sink]`. Independent of
/// [`powl2_language`] by construction (no shared code), which is what makes
/// their agreement on a converted model real evidence rather than a tautology.
/// It is *bounded* agreement, not the paper's Theorem 5.5 -- there is no
/// "Theorem 1" in Kourani/Park/van der Aalst.
///
/// Markings are [`Marking`] -- a token count per place -- not sets of place
/// names. A set-valued marking cannot hold two tokens in one place, so firing
/// a transition *removed* a place rather than decrementing its count, and on a
/// net that is not 1-safe the enumerated language came out **wrong** rather
/// than merely incomplete: it admitted firing sequences the net cannot perform
/// and missed ones it can. Since this enumeration is one half of the
/// independent-oracle check in `convert_and_verify`, a wrong language there
/// weakens the very comparison it exists to provide.
///
/// The conversion path refuses unsafe nets before reaching here, so the defect
/// was latent rather than live -- but the enumerator is public and correctness
/// should not depend on the caller having run the gate.
#[must_use]
pub fn wf_net_language(net: &WfNet, max_len: usize) -> Language {
    let mut out = Language::new();
    let places: Vec<&String> = net.places().iter().collect();
    let index: BTreeMap<&str, usize> = places
        .iter()
        .enumerate()
        .map(|(i, p)| (p.as_str(), i))
        .collect();
    let mut start: Marking = vec![0; places.len()];
    start[index[net.source()]] = 1;
    let mut sink_marking: Marking = vec![0; places.len()];
    sink_marking[index[net.sink()]] = 1;
    let mut trace = Vec::new();
    explore(
        net,
        &index,
        &start,
        &sink_marking,
        &mut trace,
        0,
        max_len,
        &mut out,
    );
    out
}

fn explore(
    net: &WfNet,
    index: &BTreeMap<&str, usize>,
    marking: &Marking,
    sink_marking: &Marking,
    trace: &mut Vec<String>,
    fired: usize,
    max_len: usize,
    out: &mut Language,
) {
    if marking == sink_marking {
        out.insert(trace.clone());
        return;
    }
    if fired >= 2 * max_len + 2 {
        return;
    }
    for t in net.transitions().keys() {
        let pre = net.pre_trans(t);
        if !pre.iter().all(|p| marking[index[p.as_str()]] >= 1) {
            continue;
        }
        let mut next = marking.clone();
        for p in &pre {
            next[index[p.as_str()]] -= 1;
        }
        for p in net.post_trans(t) {
            next[index[p.as_str()]] += 1;
        }
        let label = net.label(t);
        let pushed = label.is_some();
        if let Some(a) = label {
            if trace.len() + 1 > max_len {
                continue;
            }
            trace.push(a);
        }
        explore(
            net,
            index,
            &next,
            sink_marking,
            trace,
            fired + 1,
            max_len,
            out,
        );
        if pushed {
            trace.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn po_with_dangling_edge() -> Powl2Model {
        Powl2Model::PartialOrder {
            children: vec![
                Powl2Model::Activity("a".to_string()),
                Powl2Model::Activity("b".to_string()),
            ],
            // Child index 5 does not exist: only 0 and 1 are children.
            edges: vec![(0, 5)],
        }
    }

    /// Falsifier for the dropped-edge soundness gap. Pre-fix this returned
    /// `{[a,b], [b,a]}` -- the dangling edge silently deleted, so the
    /// enumerated language was a strict superset of the model's and could
    /// make `convert_and_verify`'s differential check pass by being weaker.
    #[test]
    fn falsifier_out_of_range_partial_order_edge_is_refused() {
        let model = po_with_dangling_edge();
        assert_eq!(
            powl2_language(&model, 4),
            Err(Powl2Error::InvalidEdge {
                from: 0,
                to: 5,
                len: 2
            })
        );
    }

    /// The in-range case still enumerates: the refusal is not a blanket
    /// rejection of partial orders.
    #[test]
    fn in_range_partial_order_edge_still_enumerates() {
        let model = Powl2Model::PartialOrder {
            children: vec![
                Powl2Model::Activity("a".to_string()),
                Powl2Model::Activity("b".to_string()),
            ],
            edges: vec![(0, 1)],
        };
        let lang = powl2_language(&model, 4).expect("in-range edges enumerate");
        assert_eq!(
            lang,
            [vec!["a".to_string(), "b".to_string()]]
                .into_iter()
                .collect::<Language>()
        );
    }
}
