//! Executable scheduler for planner-output POWL v2 tapes.
//!
//! `compiler::v2::compile_powl_v2` produces `tape::v2::PowlTape`. The legacy
//! scheduler consumes a different tape type, so planner-produced POWL models
//! previously stopped at compilation. This module is the direct execution
//! bridge: readiness is computed from the compiled predecessor relation and
//! simultaneous firing is admitted by the compiled minimal-nonface table.

use bcinr_mfw_ir::EventSet;

use crate::scheduler::ConcurrencySelector;
use crate::tape::v2::{ConcurrencyGuardTable, PowlTape};

/// Runtime state for one bounded POWL v2 execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PowlV2RunState {
    /// Operations that have completed.
    pub done_mask: u64,
    /// Logical scheduler ticks executed.
    pub tick: u32,
}

impl PowlV2RunState {
    /// Construct a fresh execution state.
    pub const fn new() -> Self {
        Self {
            done_mask: 0,
            tick: 0,
        }
    }

    /// True when every valid tape slot has completed.
    pub fn is_complete(&self, tape: &PowlTape) -> bool {
        self.done_mask & valid_mask(tape.len) == valid_mask(tape.len)
    }

    /// Compute every unfinished operation whose predecessor set is complete.
    pub fn ready_mask(&self, tape: &PowlTape) -> u64 {
        let mut ready = 0u64;
        for index in 0..tape.len as usize {
            let bit = 1u64 << index;
            let unfinished = self.done_mask & bit == 0;
            let predecessors_complete = tape.ops[index].pred_mask & !self.done_mask == 0;
            if unfinished && predecessors_complete {
                ready |= bit;
            }
        }
        ready
    }
}

/// Outcome of one scheduler tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowlV2TickOutcome {
    /// At least one operation fired; carries the tape-slot mask.
    Fired(u64),
    /// All operations had already completed.
    Complete,
    /// The tape is incomplete but no operation can fire.
    Deadlock { remaining_mask: u64 },
}

/// Execute one deterministic, concurrency-admitted POWL v2 tick.
///
/// The selector must return a subset of the ready set admitted by `guards`;
/// `ConcurrencySelector::select_checked` enforces both postconditions.
pub fn scheduler_tick_v2<S: ConcurrencySelector>(
    tape: &PowlTape,
    state: &mut PowlV2RunState,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
) -> PowlV2TickOutcome {
    // Early exit for completion (unavoidable check)
    if state.is_complete(tape) {
        return PowlV2TickOutcome::Complete;
    }

    // Compute readiness unconditionally (no branch on result)
    let ready_mask = state.ready_mask(tape);
    let ready = mask_to_event_set(ready_mask);
    let selected = selector.select_checked(&ready, guards);
    let fired = event_set_to_mask(&selected);

    // Combine both empty conditions: if either ready_mask or fired is zero, deadlock
    // This reduces two separate if statements into one conditional check
    if ready_mask == 0 || fired == 0 {
        return PowlV2TickOutcome::Deadlock {
            remaining_mask: valid_mask(tape.len) & !state.done_mask,
        };
    }

    // Only reachable if both ready_mask and fired are nonzero
    state.done_mask |= fired;
    state.tick = state.tick.saturating_add(1);
    PowlV2TickOutcome::Fired(fired)
}

/// Execute until completion, deadlock, or `max_ticks` is reached.
pub fn execute_v2<S: ConcurrencySelector>(
    tape: &PowlTape,
    selector: &mut S,
    guards: &ConcurrencyGuardTable,
    max_ticks: u32,
) -> (PowlV2RunState, PowlV2TickOutcome) {
    let mut state = PowlV2RunState::new();
    for _ in 0..max_ticks {
        let outcome = scheduler_tick_v2(tape, &mut state, selector, guards);
        match outcome {
            PowlV2TickOutcome::Fired(_) => {
                if state.is_complete(tape) {
                    return (state, PowlV2TickOutcome::Complete);
                }
            }
            PowlV2TickOutcome::Complete | PowlV2TickOutcome::Deadlock { .. } => {
                return (state, outcome);
            }
        }
    }
    let remaining_mask = valid_mask(tape.len) & !state.done_mask;
    (
        state,
        if remaining_mask == 0 {
            PowlV2TickOutcome::Complete
        } else {
            PowlV2TickOutcome::Deadlock { remaining_mask }
        },
    )
}

const fn valid_mask(len: u8) -> u64 {
    if len == 0 {
        0
    } else if len >= 64 {
        u64::MAX
    } else {
        (1u64 << len) - 1
    }
}

fn mask_to_event_set(mask: u64) -> EventSet {
    let mut set = EventSet::empty();
    let mut bits = mask;
    while bits != 0 {
        let index = bits.trailing_zeros() as usize;
        bits &= bits - 1;
        set.insert(index);
    }
    set
}

fn event_set_to_mask(set: &EventSet) -> u64 {
    let mut mask = 0u64;
    for index in set.iter_stable() {
        mask |= 1u64 << index;
    }
    mask
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use bcinr_mfw_ir::{
        ActionOccurrenceId, Digest, EventSet, ExecutableConcurrencyComplex, MinimalNonFace,
        PowlNodeId, PrecedenceEdge, StrictPartialOrder,
    };

    use crate::compiler::v2::compile_powl_v2;
    use crate::model::{ActivityNode, PowlModel, PowlNode};
    use crate::scheduler::StableMaximalSelector;
    use crate::tape::v2::{CompiledNonFace, ConcurrencyGuardTable};

    use super::*;

    fn three_node_fork_join() -> PowlModel {
        let nodes = (0..3)
            .map(|index| {
                PowlNode::Activity(ActivityNode {
                    id: PowlNodeId(index),
                    label: format!("a{index}"),
                    source_action: ActionOccurrenceId(index as u32),
                })
            })
            .collect();
        let mut edges = BTreeSet::new();
        edges.insert(PrecedenceEdge {
            before: ActionOccurrenceId(0),
            after: ActionOccurrenceId(2),
        });
        edges.insert(PrecedenceEdge {
            before: ActionOccurrenceId(1),
            after: ActionOccurrenceId(2),
        });
        let provenance = (0..3)
            .map(|index| (PowlNodeId(index), ActionOccurrenceId(index as u32)))
            .collect::<BTreeMap<_, _>>();
        PowlModel {
            nodes,
            order: StrictPartialOrder { edges },
            concurrency: ExecutableConcurrencyComplex {
                event_count: 3,
                minimal_nonfaces: vec![],
                conflict_witnesses: BTreeMap::new(),
                digest: Digest::hash(b"empty"),
            },
            provenance,
        }
    }

    #[test]
    fn compiled_v2_fork_join_executes_without_legacy_tape_conversion() {
        let compiled = compile_powl_v2(&three_node_fork_join()).unwrap();
        let mut state = PowlV2RunState::new();
        let mut selector = StableMaximalSelector;

        assert_eq!(
            scheduler_tick_v2(&compiled.tape, &mut state, &mut selector, &compiled.guards,),
            PowlV2TickOutcome::Fired(0b011)
        );
        assert_eq!(
            scheduler_tick_v2(&compiled.tape, &mut state, &mut selector, &compiled.guards,),
            PowlV2TickOutcome::Fired(0b100)
        );
        assert!(state.is_complete(&compiled.tape));
    }

    #[test]
    fn compiled_guard_defers_an_incompatible_ready_pair() {
        let compiled = compile_powl_v2(&three_node_fork_join()).unwrap();
        let pair = EventSet::empty().with(0).with(1);
        let guards = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: pair,
                witness_digest: Digest::hash(b"pair-conflict"),
            }],
        };
        let mut selector = StableMaximalSelector;
        let (state, outcome) = execute_v2(&compiled.tape, &mut selector, &guards, 4);

        assert_eq!(outcome, PowlV2TickOutcome::Complete);
        assert_eq!(state.done_mask, 0b111);
        assert_eq!(state.tick, 3);
    }

    #[test]
    fn zero_tick_budget_cannot_claim_completion_for_nonempty_tape() {
        let compiled = compile_powl_v2(&three_node_fork_join()).unwrap();
        let mut selector = StableMaximalSelector;
        let (state, outcome) = execute_v2(
            &compiled.tape,
            &mut selector,
            &ConcurrencyGuardTable::empty(),
            0,
        );
        assert_eq!(state.done_mask, 0);
        assert_eq!(
            outcome,
            PowlV2TickOutcome::Deadlock {
                remaining_mask: 0b111
            }
        );
    }

    #[test]
    fn unrelated_minimal_nonface_does_not_block_execution() {
        let compiled = compile_powl_v2(&three_node_fork_join()).unwrap();
        let guards = ConcurrencyGuardTable {
            nonfaces: vec![CompiledNonFace {
                members: EventSet::empty().with(5).with(6),
                witness_digest: Digest::hash(b"outside-tape"),
            }],
        };
        let mut selector = StableMaximalSelector;
        let (state, outcome) = execute_v2(&compiled.tape, &mut selector, &guards, 3);
        assert_eq!(outcome, PowlV2TickOutcome::Complete);
        assert_eq!(state.tick, 2);
    }

    #[test]
    fn source_complex_fixture_can_be_used_directly() {
        let model = three_node_fork_join();
        let _source_nonfaces: Vec<MinimalNonFace> = model.concurrency.minimal_nonfaces.clone();
        let compiled = compile_powl_v2(&model).unwrap();
        assert_eq!(compiled.tape.len, 3);
    }
}
