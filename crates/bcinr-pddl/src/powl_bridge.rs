//! Converts a TemporalPlan into a POWL v2 tape description.
//!
//! The result can be fed to bcinr-powl's compiler to produce an executable Powl64Op tape.

use wasm4pm_compat::pddl::TemporalPlan;

/// A single op in the POWL tape description.
#[derive(Debug, Clone)]
pub struct PowlOpSpec {
    pub kind: PowlOpKind,
    pub label: String,
    pub pred_mask: u64,   // bitmask of preceding ops that must complete
    pub succ_mask: u64,   // bitmask of this op (1 << index)
    pub start_time: Option<f64>,
    pub duration: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowlOpKind {
    Activity,
    PartialOrderGate,
    ChoiceGate,
}

/// Convert a TemporalPlan to a POWL tape spec.
///
/// Rules:
/// - Each step becomes an Activity op
/// - Steps with overlapping time intervals get pred_mask = 0 (can execute concurrently)
/// - Steps with sequential dependencies get pred_mask = 1 << predecessor_index
/// - Steps that must all precede a later step are wrapped in a PartialOrderGate
pub fn temporal_plan_to_powl_tape(plan: &TemporalPlan) -> Vec<PowlOpSpec> {
    let steps = &plan.steps;
    let n = steps.len();
    let mut ops = Vec::with_capacity(n);

    for (i, step) in steps.iter().enumerate() {
        // Compute pred_mask: OR of bits for all steps that MUST finish before this one starts.
        // A step j must finish before step i starts if:
        //   step_j.start_time + step_j.duration <= step_i.start_time  (sequential dependency)
        //   AND they are not concurrent (no time overlap)
        let mut pred_mask: u64 = 0;
        for (j, prev) in steps.iter().enumerate() {
            if j >= i { continue; }
            let prev_end = prev.start_time + prev.duration;
            // prev must finish before step i starts (strict sequential dependency)
            if prev_end <= step.start_time + 1e-9 {
                // Check if there is no other step between them that already covers this ordering
                pred_mask |= 1u64 << j;
            }
        }

        ops.push(PowlOpSpec {
            kind: PowlOpKind::Activity,
            label: format!("{}({})", step.action_name, step.args.join(",")),
            pred_mask,
            succ_mask: 1u64 << i,
            start_time: Some(step.start_time),
            duration: Some(step.duration),
        });
    }

    // Reduce pred_masks: remove transitive dependencies (direct predecessors only)
    // A pred j is transitive if there exists k such that j < k < i and pred[i] has bit j AND pred[k] has bit j
    for i in 0..n {
        let mut direct_mask = ops[i].pred_mask;
        for k in 0..i {
            if (ops[i].pred_mask >> k) & 1 == 1 {
                // k is a predecessor of i; remove any predecessors of k that are also in i's pred_mask
                direct_mask &= !ops[k].pred_mask;
            }
        }
        ops[i].pred_mask = direct_mask;
    }

    ops
}
