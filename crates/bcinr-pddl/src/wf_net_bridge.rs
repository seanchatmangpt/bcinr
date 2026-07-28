//! Stage-2 bridge: materialize a [`bcinr_mfw_ir::CausalPlan`] (the causal
//! partial order [`crate::causal::PddlCausalAnalyzer`] computes over a flat
//! PDDL plan) into an actual `bcinr_powl::wf_net::WfNet`, then run it through
//! `bcinr_powl::wf_to_powl::convert` (Algorithm 3) to recover genuinely
//! hierarchical POWL 2.0 structure.
//!
//! Confirmed missing everywhere else in the ecosystem this session: every
//! existing WF-net -> POWL decomposition (wasm4pm, `powl2-decompose`) has
//! only ever been fed a hand-built or POWL-round-tripped net, never a
//! PDDL-plan-derived one, and `bcinr-pddl/src/cognitive.rs`'s own
//! PDDL -> POWL projection stays flat/sequential by design, deferring
//! concurrency to "a separate rail" -- this module is that rail.

use bcinr_mfw_ir::{ActionPair, CausalPlan};
use bcinr_powl::powl2::Powl2Model;
use bcinr_powl::wf_net::WfNet;
use bcinr_powl::wf_to_powl::{convert_and_verify, Refusal, RefusalReason, DEFAULT_DEPTH_BUDGET};

use crate::capability::GroundedPlanningEpoch;

/// Build a flat `Powl2Model::PartialOrder` over the plan's occurrences,
/// ordered by the causal analysis's `dependent` relation -- independent
/// pairs are left unordered so they can run concurrently once recomposed.
fn causal_plan_to_partial_order(
    epoch: &GroundedPlanningEpoch,
    causal_plan: &CausalPlan,
) -> Powl2Model {
    let occurrences = &causal_plan.occurrences;
    if occurrences.is_empty() {
        return Powl2Model::Silent;
    }

    let children: Vec<Powl2Model> = occurrences
        .iter()
        .map(|occ| {
            epoch
                .actions
                .get(occ.action as usize)
                .map(|a| Powl2Model::Activity(a.label.clone()))
                .unwrap_or(Powl2Model::Silent)
        })
        .collect();

    if children.len() == 1 {
        return children.into_iter().next().expect("length checked");
    }

    let mut edges = Vec::new();
    for i in 0..occurrences.len() {
        for j in (i + 1)..occurrences.len() {
            let pair = ActionPair::new(occurrences[i].id, occurrences[j].id);
            if causal_plan.independence.dependent.contains_key(&pair) {
                edges.push((i, j));
            }
        }
    }

    Powl2Model::PartialOrder { children, edges }
}

/// Materialize a causal plan as an actual `WfNet` via recomposition. Errs
/// (rather than panics) if recomposition's own algorithm-internal
/// `WfNet::new` check fails -- see `bcinr_powl::recompose::recompose`'s doc
/// comment.
pub fn causal_plan_to_wf_net(
    epoch: &GroundedPlanningEpoch,
    causal_plan: &CausalPlan,
) -> Result<WfNet, bcinr_powl::wf_net::NetError> {
    let model = causal_plan_to_partial_order(epoch, causal_plan);
    bcinr_powl::recompose::recompose(&model)
}

/// The full Stage-2 bridge: causal analysis -> `WfNet` -> Algorithm 3
/// decomposition. Returns genuinely hierarchical POWL 2.0 (nested
/// `PartialOrder`/`ChoiceGraph`) where the causal structure supports it,
/// rather than the flat sequence `cognitive.rs`'s own projection produces.
///
/// Gated by *bounded* language agreement via [`convert_and_verify`], not bare
/// `convert`: the returned model's denotational language must equal the
/// WF-net's own token-game replay up to the checked bound, or the conversion
/// is refused (`RefusalReason::BoundedLanguageAgreementFailed`). This is not
/// the paper's Theorem 5.5 -- see `convert_and_verify`'s own docs for what the
/// bound does and does not establish. A POWL model that is *not*
/// language-equivalent to the plan it claims to describe is worse than no
/// model at all, because it still looks authoritative -- so the check that
/// makes the claim true runs on the production path, not only in tests.
///
/// `max_len` is the **recomposed net's transition count**, not the plan's
/// occurrence count. The occurrence count bounds the number of *visible*
/// activities in a trace, but `wf_net_language`'s replay derives its total
/// firing budget from `max_len` (`2 * max_len + 2`), and `recompose` inserts
/// several silent tau gates per child (`po_init`/`po_go`/`po_fin`/`po_fini`,
/// `seq_gate`, ...) -- so a bound sized to visible activities alone starves
/// the replay before it reaches the sink and compares a truncated language
/// against a complete one, refusing a perfectly good model. The transition
/// count is the exact right bound here: the net is recomposed from a
/// `PartialOrder`, hence acyclic, so no firing sequence can fire more
/// transitions than exist, making the enumerated language complete.
pub fn causal_plan_to_powl2(
    epoch: &GroundedPlanningEpoch,
    causal_plan: &CausalPlan,
) -> Result<Powl2Model, Refusal> {
    let net = causal_plan_to_wf_net(epoch, causal_plan).map_err(|err| Refusal {
        reason: RefusalReason::InternalNetConstruction(err),
        // No `WfNet` was ever successfully constructed to hash -- mirrors
        // `capability.rs`'s `Digest::ZERO` convention for a refusal raised
        // before any real content-derived digest can exist.
        net_hash: "0".repeat(64),
    })?;
    let max_len = net.transitions().len();
    convert_and_verify(&net, DEFAULT_DEPTH_BUDGET, max_len)
}
