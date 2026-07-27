//! POWL 2.0 -> WF-net recomposition, ported and adapted from
//! `~/ggen/crates/powl2-decompose/src/recompose.rs` (MIT OR Apache-2.0,
//! © Sean Chatman, same copyright holder as this crate). Used both as a
//! round-trip differential oracle (`L(recompose(convert(N))) == L(N)`) and as
//! the Stage-2 bridge: a flat `Powl2Model::PartialOrder` built from a causal
//! analysis can be recomposed into a `WfNet` and re-decomposed via
//! [`crate::wf_to_powl::convert`] to discover genuine hierarchical structure.

use std::collections::{BTreeMap, BTreeSet};

use crate::powl2::Powl2Model;
use crate::wf_net::{Label, NetError, WfNet};

/// Recompose a `Powl2Model` into an equivalent safe & sound WF-net. Errs if
/// the recomposition algorithm produces a candidate that fails `WfNet::new`'s
/// own well-formedness check -- an algorithm-internal inconsistency rather
/// than a property of `model`, but surfaced as a typed error instead of a
/// panic so it cannot crash a caller on adversarial-but-valid input.
pub fn recompose(model: &Powl2Model) -> Result<WfNet, NetError> {
    let mut b = Builder::default();
    let (src, snk) = b.build(model);
    b.finish(src, snk)
}

#[derive(Default)]
struct Builder {
    counter: usize,
    places: BTreeSet<String>,
    transitions: BTreeMap<String, Label>,
    pt: BTreeSet<(String, String)>,
    tp: BTreeSet<(String, String)>,
}

impl Builder {
    fn id(&mut self, stem: &str) -> String {
        let id = format!("{stem}#{}", self.counter);
        self.counter += 1;
        id
    }
    fn place(&mut self, stem: &str) -> String {
        let id = self.id(stem);
        self.places.insert(id.clone());
        id
    }
    fn trans(&mut self, stem: &str, label: Label) -> String {
        let id = self.id(stem);
        self.transitions.insert(id.clone(), label);
        id
    }
    fn arc_pt(&mut self, p: &str, t: &str) {
        self.pt.insert((p.to_string(), t.to_string()));
    }
    fn arc_tp(&mut self, t: &str, p: &str) {
        self.tp.insert((t.to_string(), p.to_string()));
    }

    fn build(&mut self, model: &Powl2Model) -> (String, String) {
        match model {
            Powl2Model::Activity(label) => self.build_leaf(Some(label.clone())),
            Powl2Model::Silent => self.build_leaf(None),
            Powl2Model::Sequence(children) => self.build_sequence(children),
            Powl2Model::PartialOrder { children, edges } => {
                self.build_partial_order(children, edges)
            }
            Powl2Model::ChoiceGraph {
                children,
                edges,
                start,
                end,
            } => self.build_choice(children, edges, *start, *end),
            Powl2Model::DoRedo { body, redo, .. } => self.build_do_redo(body, redo),
        }
    }

    fn build_leaf(&mut self, label: Label) -> (String, String) {
        let s = self.place("s");
        let k = self.place("k");
        let t = self.trans("t", label);
        self.arc_pt(&s, &t);
        self.arc_tp(&t, &k);
        (s, k)
    }

    fn build_sequence(&mut self, children: &[Powl2Model]) -> (String, String) {
        if children.is_empty() {
            return self.build_leaf(None);
        }
        let (first_s, mut prev_k) = self.build(&children[0]);
        for child in &children[1..] {
            let (s_i, k_i) = self.build(child);
            let gate = self.trans("seq_gate", None);
            self.arc_pt(&prev_k, &gate);
            self.arc_tp(&gate, &s_i);
            prev_k = k_i;
        }
        (first_s, prev_k)
    }

    fn build_partial_order(
        &mut self,
        children: &[Powl2Model],
        order: &[(usize, usize)],
    ) -> (String, String) {
        let n = children.len();
        let order_set: BTreeSet<(usize, usize)> = order.iter().copied().collect();
        let subs: Vec<(String, String)> = children.iter().map(|c| self.build(c)).collect();
        let cover = cover_relation(&order_set, n);

        let s = self.place("po_s");
        let k = self.place("po_k");
        let init = self.trans("po_init", None);
        let fini = self.trans("po_fini", None);
        self.arc_pt(&s, &init);
        self.arc_tp(&fini, &k);

        let ready: Vec<String> = (0..n).map(|_| self.place("po_ready")).collect();
        let post: Vec<String> = (0..n).map(|_| self.place("po_post")).collect();
        let mut edge_place: BTreeMap<(usize, usize), String> = BTreeMap::new();
        for &(j, i) in &cover {
            edge_place.insert((j, i), self.place("po_edge"));
        }

        for i in 0..n {
            self.arc_tp(&init, &ready[i]);
            let go = self.trans("po_go", None);
            self.arc_pt(&ready[i], &go);
            for (&(j, ii), ep) in &edge_place {
                if ii == i {
                    let _ = j;
                    self.arc_pt(ep, &go);
                }
            }
            self.arc_tp(&go, &subs[i].0);

            let fin = self.trans("po_fin", None);
            self.arc_pt(&subs[i].1, &fin);
            self.arc_tp(&fin, &post[i]);
            for (&(j, ii), ep) in &edge_place {
                if j == i {
                    let _ = ii;
                    self.arc_tp(&fin, ep);
                }
            }
            self.arc_pt(&post[i], &fini);
        }

        (s, k)
    }

    fn build_choice(
        &mut self,
        children: &[Powl2Model],
        edges: &[(usize, usize)],
        start: usize,
        end: usize,
    ) -> (String, String) {
        let subs: Vec<(String, String)> = children.iter().map(|c| self.build(c)).collect();
        let s = self.place("cg_s");
        let k = self.place("cg_k");

        let out_place = |node: usize| -> String {
            if node == start {
                s.clone()
            } else if node == end {
                k.clone()
            } else {
                subs[node].1.clone()
            }
        };
        let in_place = |node: usize| -> String {
            if node == start {
                s.clone()
            } else if node == end {
                k.clone()
            } else {
                subs[node].0.clone()
            }
        };

        for &(u, v) in edges {
            let from = out_place(u);
            let to = in_place(v);
            let e = self.trans("cg_e", None);
            self.arc_pt(&from, &e);
            self.arc_tp(&e, &to);
        }

        (s, k)
    }

    /// Do-redo as an unbounded cycle: `enter -> body -> exit`, plus
    /// `body -> redo -> body` re-entry. `max_redos` is not enforced as a
    /// hard iteration count here -- a WF-net cycle has no counter -- the
    /// caller-facing bound is the depth/state-space budgets `convert`/
    /// `language_upto` already carry.
    fn build_do_redo(&mut self, body: &Powl2Model, redo: &Powl2Model) -> (String, String) {
        let (s_b, k_b) = self.build(body);
        let (s_r, k_r) = self.build(redo);

        let s = self.place("dr_s");
        let k = self.place("dr_k");
        let enter = self.trans("dr_enter", None);
        let exit = self.trans("dr_exit", None);
        let redo_in = self.trans("dr_redo_in", None);
        let redo_out = self.trans("dr_redo_out", None);

        self.arc_pt(&s, &enter);
        self.arc_tp(&enter, &s_b);
        self.arc_pt(&k_b, &exit);
        self.arc_tp(&exit, &k);
        self.arc_pt(&k_b, &redo_in);
        self.arc_tp(&redo_in, &s_r);
        self.arc_pt(&k_r, &redo_out);
        self.arc_tp(&redo_out, &s_b);

        (s, k)
    }

    fn finish(self, source: String, sink: String) -> Result<WfNet, NetError> {
        WfNet::new(
            self.places,
            self.transitions,
            self.pt,
            self.tp,
            source,
            sink,
        )
    }
}

/// Cover (Hasse) relation of a (possibly not transitively closed) strict
/// order: `(i,j)` with no intermediate `k` such that `i -> k -> j`.
fn cover_relation(order: &BTreeSet<(usize, usize)>, n: usize) -> BTreeSet<(usize, usize)> {
    let mut cover = BTreeSet::new();
    for &(i, j) in order {
        let has_mid =
            (0..n).any(|k| k != i && k != j && order.contains(&(i, k)) && order.contains(&(k, j)));
        if !has_mid {
            cover.insert((i, j));
        }
    }
    cover
}
