//! BCINR-CMCA-G — Allocation Identity and Replay Closure.
//!
//! BCINR-CMCA-F proved a caller can construct a CMCA-derived selector and
//! run it through the real production entrypoint (`PddlPowlPlan::
//! execute_with_selector`), and that inverting the underlying masses changes
//! the resulting fired trace. It did not prove that the *receipt* produced
//! identifies which allocation law, masses, capacity, or priority map
//! governed that execution -- two different priority maps that happen to
//! induce the same admission order were, before this module, observationally
//! indistinguishable at the receipt level, and verification depended on the
//! caller reconstructing an "equivalent" selector by hand.
//!
//! This module is the one canonical production surface for CMCA allocation:
//! [`CmcaExecutionRequest`] names every admitted input, [`allocate_pddl_powl_plan`]
//! is the one production mapping from real PDDL action identity to POWL node
//! to tape slot to CMCA priority (replacing the fixture-local mapping logic
//! BCINR-CMCA-F had to hand-roll), [`PddlPowlPlan::execute_with_cmca`] owns
//! both seal and verify selector construction so a caller can no longer
//! supply a mismatched verifier, and [`verify_cmca_execution`] independently
//! recomputes the entire allocation rather than trusting a caller-supplied
//! selector or comparing only the fired trace.
//!
//! # Governing law
//!
//! `fired trace equal` does NOT imply `allocation receipt equal`. Two
//! admitted priority maps that happen to produce the same fired trace under
//! a given capacity remain different governing artifacts and must have
//! different receipt identities (`cmca_g_allocation_identity_and_replay_closure.rs`,
//! fixture B).
//!
//! # Explicit exclusions
//!
//! No Lean correspondence claim (`AllocationSemantics::UniformSiblingCoverageQ0`
//! names only the current Rust runtime semantics), no branchless cascade
//! hardening, no CLI/MCP exposure, no signatures or remote verification.

use std::collections::BTreeMap;

use bcinr_cmca::cascade::{consequence_mass_traced, AllocationTrace, CascadeRefusal, CascadeTree};
use bcinr_cmca::fixed::NonNegativeFixed;
use bcinr_mfw_ir::{Digest, PowlNodeId};

use crate::production::{action_for_slot, PddlPowlError, PddlPowlExecution, PddlPowlPlan};

/// Names one CMCA execution profile -- e.g. `"BCINR_CMCA_PROFILE_V0_1"`.
/// Deliberately a typed identity, not free text: two profiles with
/// different identities are different governing artifacts even when they
/// happen to produce numerically equal priorities (see
/// `ProfileDigestMismatch`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProfileIdentity(pub String);

/// `lenses[d]` applies at cascade depth `d`, mirroring
/// [`bcinr_powl::multifractal::consequence_mass`]'s `&[i32]` convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LensSchedule(pub Vec<i32>);

/// Names the ONE allocation semantics this checkpoint claims. This is
/// explicitly a Rust-runtime-only claim -- BCINR-CMCA-H (not this
/// checkpoint) is where any Lean correspondence would be established, and
/// per BCINR-CMCA-H's own scoping note the correct comparison target is
/// Lean `uniformSiblingCoverage`, not Lean `escort .coverage` (a different,
/// support-coverage semantics this variant does not claim).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationSemantics {
    /// `q = 0`: uniform sibling coverage, as implemented by
    /// `bcinr_cmca::cascade::consequence_mass`/`consequence_mass_traced`
    /// today. Fixed-point cascade execution is subconservative; residuals
    /// are explicit (`AllocationStep::residual_bits`), never silently
    /// absorbed.
    UniformSiblingCoverageQ0,
}

/// One admitted CMCA execution profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmcaExecutionProfile {
    pub identity: ProfileIdentity,
    pub lens_schedule: LensSchedule,
    pub allocation_semantics: AllocationSemantics,
}

impl CmcaExecutionProfile {
    /// Deterministic commitment to every field -- two profiles with the
    /// same numeric lens schedule but different identity or semantics
    /// digest differently, on purpose (semantic identity must not collapse
    /// into coincidentally equal output).
    pub fn digest(&self) -> Digest {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"bcinr:cmca-g:profile:v1");
        buf.extend_from_slice(self.identity.0.as_bytes());
        buf.push(0xff);
        buf.extend_from_slice(&(self.lens_schedule.0.len() as u64).to_le_bytes());
        for lens in &self.lens_schedule.0 {
            buf.extend_from_slice(&lens.to_le_bytes());
        }
        buf.push(match self.allocation_semantics {
            AllocationSemantics::UniformSiblingCoverageQ0 => 0u8,
        });
        Digest::hash(&buf)
    }
}

/// Admitted input mass per real PDDL action label -- deliberately keyed by
/// the real action label (the identity chain BCINR-CMCA-F established:
/// POWL node -> provenance occurrence -> causal-plan occurrence -> epoch
/// action -> real PDDL label), never by `CompiledPowlV2::node_labels`'s
/// synthetic `"action-N"` placeholders (confirmed synthetic by BCINR-CMCA-F).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProcessMassField(pub BTreeMap<String, NonNegativeFixed>);

impl ProcessMassField {
    pub fn digest(&self) -> Digest {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"bcinr:cmca-g:mass-field:v1");
        buf.extend_from_slice(&(self.0.len() as u64).to_le_bytes());
        for (label, mass) in &self.0 {
            buf.extend_from_slice(label.as_bytes());
            buf.push(0xff);
            buf.extend_from_slice(&mass.val.to_le_bytes());
            buf.extend_from_slice(&mass.err.to_le_bytes());
        }
        Digest::hash(&buf)
    }
}

/// Every admitted input required to manufacture a CMCA-prioritized
/// execution -- the canonical request type BCINR-CMCA-F's fixture had no
/// single object for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmcaExecutionRequest {
    pub profile: CmcaExecutionProfile,
    pub capacity: u32,
    pub masses: ProcessMassField,
}

/// Distinguishable mapping/allocation failures. Reuses
/// [`bcinr_cmca::cascade::CascadeRefusal`] for genuine cascade refusals
/// rather than collapsing them into a generic variant.
#[derive(Debug, Clone, PartialEq)]
pub enum CmcaAllocationRefusal {
    EmptyCapacity,
    EmptyMassField,
    MissingActionMass { action: String },
    DuplicateActionMass { action: String },
    UnknownActionMass { action: String },
    MissingProvenance { node: PowlNodeId },
    AmbiguousProvenance { node: PowlNodeId },
    MissingOccurrence { occurrence: u32 },
    MissingAction { action_index: u64 },
    DuplicateTapeBinding { slot: usize },
    UnboundTapeSlot { slot: usize },
    UnsupportedLens,
    AllocationRefused { source: CascadeRefusal },
    ProcessDigestMismatch,
    ProfileDigestMismatch,
    PriorityDigestMismatch,
}

impl std::fmt::Display for CmcaAllocationRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for CmcaAllocationRefusal {}

impl From<CascadeRefusal> for CmcaAllocationRefusal {
    fn from(source: CascadeRefusal) -> Self {
        Self::AllocationRefused { source }
    }
}

/// The standing-bearing allocation artifact: every input and every derived
/// value that governed one CMCA-prioritized execution, bound together.
/// Constructed exactly once, by [`allocate_pddl_powl_plan`] -- never
/// assembled ad hoc by a caller.
#[derive(Debug, Clone)]
pub struct CmcaAllocatedExecution {
    pub process_digest: Digest,
    pub tape_root: Digest,
    pub profile_identity: ProfileIdentity,
    pub profile_digest: Digest,
    pub mass_field_digest: Digest,
    pub capacity: u32,
    /// Tape-slot-id -> admitted CMCA priority. Canonical, not caller-built.
    pub priority_map: BTreeMap<usize, NonNegativeFixed>,
    pub priority_digest: Digest,
    pub allocation_trace: AllocationTrace,
    pub allocation_trace_digest: Digest,
}

fn digest_process(plan: &PddlPowlPlan) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"bcinr:cmca-g:process:v1");
    let mut occurrences: Vec<_> = plan.workflow.causal_plan.occurrences.iter().collect();
    occurrences.sort_by_key(|occurrence| occurrence.id.0);
    buf.extend_from_slice(&(occurrences.len() as u64).to_le_bytes());
    for occurrence in occurrences {
        buf.extend_from_slice(&occurrence.id.0.to_le_bytes());
        buf.extend_from_slice(&occurrence.action.to_le_bytes());
        if let Some(action) = plan.workflow.epoch.actions.get(occurrence.action as usize) {
            buf.extend_from_slice(action.label.as_bytes());
        }
        buf.push(0xff);
    }
    Digest::hash(&buf)
}

fn digest_priority_map(priority_map: &BTreeMap<usize, NonNegativeFixed>) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"bcinr:cmca-g:priority:v1");
    buf.extend_from_slice(&(priority_map.len() as u64).to_le_bytes());
    for (&slot, mass) in priority_map {
        buf.extend_from_slice(&(slot as u64).to_le_bytes());
        buf.extend_from_slice(&mass.val.to_le_bytes());
        buf.extend_from_slice(&mass.err.to_le_bytes());
    }
    Digest::hash(&buf)
}

fn digest_trace(trace: &AllocationTrace) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"bcinr:cmca-g:trace:v1");
    buf.extend_from_slice(&(trace.steps.len() as u64).to_le_bytes());
    for step in &trace.steps {
        buf.extend_from_slice(&(step.node as u64).to_le_bytes());
        buf.extend_from_slice(
            &step
                .parent
                .map(|p| p as u64)
                .unwrap_or(u64::MAX)
                .to_le_bytes(),
        );
        buf.extend_from_slice(&(step.wave as u64).to_le_bytes());
        buf.extend_from_slice(&step.input_share.val.to_le_bytes());
        buf.extend_from_slice(&step.input_share.err.to_le_bytes());
        buf.extend_from_slice(&(step.child_shares.len() as u64).to_le_bytes());
        for (child, contribution) in &step.child_shares {
            buf.extend_from_slice(&(*child as u64).to_le_bytes());
            buf.extend_from_slice(&contribution.val.to_le_bytes());
            buf.extend_from_slice(&contribution.err.to_le_bytes());
        }
        buf.extend_from_slice(&step.child_sum.val.to_le_bytes());
        buf.extend_from_slice(&step.child_sum.err.to_le_bytes());
        buf.extend_from_slice(&step.residual_bits.to_le_bytes());
    }
    buf.extend_from_slice(&(trace.leaves.len() as u64).to_le_bytes());
    for leaf in &trace.leaves {
        buf.extend_from_slice(&leaf.val.to_le_bytes());
        buf.extend_from_slice(&leaf.err.to_le_bytes());
    }
    Digest::hash(&buf)
}

/// Walk `plan`'s real identity chain (POWL node -> provenance occurrence ->
/// causal-plan occurrence -> epoch action -> real PDDL label) and return
/// `(tape_slot, label)` for every production action, in ascending tape-slot
/// order. This is the ONE production version of the mapping BCINR-CMCA-F's
/// fixture had to hand-roll (`action_for_slot`'s chain, walked here instead
/// of duplicated).
fn real_action_labels(plan: &PddlPowlPlan) -> Result<Vec<(usize, String)>, CmcaAllocationRefusal> {
    let mut seen_slots: BTreeMap<usize, String> = BTreeMap::new();
    for &node_id in plan.workflow.powl_model.provenance.keys() {
        let slot = node_id.0 as usize;
        // Reuse production.rs's own canonical chain walk instead of
        // hand-rolling a second copy of it (BCINR-CMCA-F's fixture had to;
        // this is the one production surface that doesn't).
        let action = action_for_slot(&plan.workflow, slot).map_err(|error| match error {
            PddlPowlError::MissingProvenance { node } => CmcaAllocationRefusal::MissingProvenance {
                node: bcinr_mfw_ir::PowlNodeId(node),
            },
            PddlPowlError::MissingOccurrence { occurrence } => {
                CmcaAllocationRefusal::MissingOccurrence { occurrence }
            }
            PddlPowlError::ActionIndexOutOfRange { action_index } => {
                CmcaAllocationRefusal::MissingAction { action_index }
            }
            // `action_for_slot` only ever returns `MissingProvenance`,
            // `MissingOccurrence`, or `ActionIndexOutOfRange` (see its
            // implementation in `production.rs`) -- both matched above.
            other => unreachable!("action_for_slot returned an unexpected variant: {other}"),
        })?;
        if let Some(existing) = seen_slots.get(&slot) {
            if existing != &action.label {
                return Err(CmcaAllocationRefusal::DuplicateTapeBinding { slot });
            }
        } else {
            seen_slots.insert(slot, action.label.clone());
        }
    }
    if seen_slots.is_empty() {
        return Err(CmcaAllocationRefusal::EmptyMassField);
    }
    Ok(seen_slots.into_iter().collect())
}

/// The one canonical production mapping from a planned PDDL/POWL process
/// plus admitted CMCA inputs to a bound, standing-bearing allocation
/// artifact. Every executable PDDL action must receive exactly one admitted
/// mass -- missing, duplicate, or unknown masses refuse (typed), they never
/// silently fall back to [`NonNegativeFixed::ZERO`].
pub fn allocate_pddl_powl_plan(
    plan: &PddlPowlPlan,
    request: &CmcaExecutionRequest,
) -> Result<CmcaAllocatedExecution, CmcaAllocationRefusal> {
    if request.capacity == 0 {
        return Err(CmcaAllocationRefusal::EmptyCapacity);
    }
    if request.masses.0.is_empty() {
        return Err(CmcaAllocationRefusal::EmptyMassField);
    }

    let slots = real_action_labels(plan)?;

    // Every production action requires exactly one admitted mass; every
    // admitted mass must be consumed by a real production action.
    let mut used_labels: BTreeMap<&str, ()> = BTreeMap::new();
    for (_, label) in &slots {
        if used_labels.insert(label.as_str(), ()).is_some() {
            return Err(CmcaAllocationRefusal::DuplicateActionMass {
                action: label.clone(),
            });
        }
        if !request.masses.0.contains_key(label) {
            return Err(CmcaAllocationRefusal::MissingActionMass {
                action: label.clone(),
            });
        }
    }
    for label in request.masses.0.keys() {
        if !used_labels.contains_key(label.as_str()) {
            return Err(CmcaAllocationRefusal::UnknownActionMass {
                action: label.clone(),
            });
        }
    }

    // Real cascade: a flat tree, root at index 0, one leaf child per
    // production action in ascending-tape-slot order (index i+1).
    let n = slots.len();
    let (priority_map, allocation_trace) = if n == 1 {
        // consequence_mass's own convention: the root is the sole member
        // of its sibling group and gets NonNegativeFixed::ONE -- no cascade
        // arithmetic needed for a single action.
        let mut priority_map = BTreeMap::new();
        priority_map.insert(slots[0].0, NonNegativeFixed::ONE);
        (
            priority_map,
            AllocationTrace {
                steps: Vec::new(),
                leaves: vec![NonNegativeFixed::ONE],
            },
        )
    } else {
        let mut parent = vec![None];
        let mut mass = vec![NonNegativeFixed::ONE];
        for (_, label) in &slots {
            parent.push(Some(0usize));
            mass.push(request.masses.0[label]);
        }
        let tree = CascadeTree::new(parent, mass)?;
        let trace = consequence_mass_traced(&tree, &request.profile.lens_schedule.0)?;
        let mut priority_map = BTreeMap::new();
        for (index, (slot, _)) in slots.iter().enumerate() {
            let leaf_mass = trace
                .leaves
                .get(index + 1)
                .copied()
                .ok_or(CmcaAllocationRefusal::UnboundTapeSlot { slot: *slot })?;
            priority_map.insert(*slot, leaf_mass);
        }
        (priority_map, trace)
    };

    let tape_root =
        Digest::hash(bcinr_powl_receipt::execution_v2::digest_tape(&plan.compiled.tape).as_bytes());
    let process_digest = digest_process(plan);
    let profile_digest = request.profile.digest();
    let mass_field_digest = request.masses.digest();
    let priority_digest = digest_priority_map(&priority_map);
    let allocation_trace_digest = digest_trace(&allocation_trace);

    Ok(CmcaAllocatedExecution {
        process_digest,
        tape_root,
        profile_identity: request.profile.identity.clone(),
        profile_digest,
        mass_field_digest,
        capacity: request.capacity,
        priority_map,
        priority_digest,
        allocation_trace,
        allocation_trace_digest,
    })
}

/// Everything [`CmcaAllocatedExecution`] commits to, in receipt form.
#[derive(Debug, Clone)]
pub struct CmcaAllocationReceipt {
    pub process_digest: Digest,
    pub tape_root: Digest,
    pub profile_identity: ProfileIdentity,
    pub profile_digest: Digest,
    pub mass_field_digest: Digest,
    pub capacity: u32,
    pub priority_digest: Digest,
    pub allocation_trace_digest: Digest,
}

impl From<&CmcaAllocatedExecution> for CmcaAllocationReceipt {
    fn from(allocation: &CmcaAllocatedExecution) -> Self {
        Self {
            process_digest: allocation.process_digest,
            tape_root: allocation.tape_root,
            profile_identity: allocation.profile_identity.clone(),
            profile_digest: allocation.profile_digest,
            mass_field_digest: allocation.mass_field_digest,
            capacity: allocation.capacity,
            priority_digest: allocation.priority_digest,
            allocation_trace_digest: allocation.allocation_trace_digest,
        }
    }
}

/// The complete CMCA-prioritized production execution receipt: allocation
/// identity, POWL v2 execution receipt, and PDDL state receipt, bound by one
/// root digest.
#[derive(Debug, Clone)]
pub struct CmcaPddlPowlExecutionReceipt {
    pub allocation: CmcaAllocationReceipt,
    pub execution: bcinr_powl_receipt::execution_v2::PowlV2ExecutionReceipt,
    pub state: crate::production::PddlPowlStateReceipt,
    pub root: Digest,
}

/// Result of [`PddlPowlPlan::execute_with_cmca`]: the bound allocation
/// artifact, the sealed receipt, and the underlying PDDL/POWL execution.
/// Not `Clone` -- [`PddlPowlExecution`] itself is not `Clone`.
#[derive(Debug)]
pub struct CmcaPddlPowlExecution {
    pub allocation: CmcaAllocatedExecution,
    pub receipt: CmcaPddlPowlExecutionReceipt,
    pub execution: PddlPowlExecution,
}

fn root_digest(
    receipt: &CmcaAllocationReceipt,
    execution_digest: Digest,
    state_digest: Digest,
) -> Digest {
    let allocation_digest = receipt
        .process_digest
        .mix(&receipt.tape_root)
        .mix(&receipt.profile_digest)
        .mix(&receipt.mass_field_digest)
        .mix(&receipt.priority_digest)
        .mix(&receipt.allocation_trace_digest)
        .mix(&Digest::hash(&receipt.capacity.to_le_bytes()));
    Digest::ZERO
        .mix(&allocation_digest)
        .mix(&execution_digest)
        .mix(&state_digest)
}

fn digest_powl_execution_receipt(
    receipt: &bcinr_powl_receipt::execution_v2::PowlV2ExecutionReceipt,
) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"bcinr:cmca-g:powl-execution:v1");
    buf.extend_from_slice(receipt.tape_root.as_bytes());
    buf.extend_from_slice(receipt.guard_root.as_bytes());
    buf.extend_from_slice(&(receipt.fired_masks.len() as u64).to_le_bytes());
    for mask in &receipt.fired_masks {
        buf.extend_from_slice(&mask.to_le_bytes());
    }
    buf.extend_from_slice(&receipt.final_done_mask.to_le_bytes());
    buf.extend_from_slice(receipt.chain_root.as_bytes());
    Digest::hash(&buf)
}

fn digest_state_receipt(receipt: &crate::production::PddlPowlStateReceipt) -> Digest {
    let mut buf = Vec::new();
    buf.extend_from_slice(b"bcinr:cmca-g:state:v1");
    buf.extend_from_slice(receipt.initial_state_root.as_bytes());
    buf.extend_from_slice(receipt.final_state_root.as_bytes());
    buf.extend_from_slice(receipt.goal_root.as_bytes());
    buf.push(receipt.goal_reached as u8);
    buf.extend_from_slice(receipt.chain_root.as_bytes());
    Digest::hash(&buf)
}

impl PddlPowlPlan {
    /// The canonical CMCA execution entrypoint -- BCINR-CMCA-G's closure of
    /// BCINR-CMCA-F's gap. Unlike [`Self::execute_with_selector`], the
    /// caller does not supply a selector at all: `allocate_pddl_powl_plan`
    /// manufactures the one canonical [`CmcaAllocatedExecution`], and both
    /// the seal and verify selectors are constructed from it internally, so
    /// a caller cannot supply a mismatched verifier.
    ///
    /// This does not change [`Self::execute`] or [`Self::execute_with_selector`]'s
    /// behavior or any existing caller of them.
    pub fn execute_with_cmca(
        self,
        request: &CmcaExecutionRequest,
    ) -> Result<CmcaPddlPowlExecution, PddlPowlError> {
        let allocation = allocate_pddl_powl_plan(&self, request).map_err(PddlPowlError::Cmca)?;

        let mut seal_selector = bcinr_powl::scheduler::PriorityCapacitySelector {
            capacity: allocation.capacity,
            priority: allocation.priority_map.clone(),
        };
        let mut verify_selector = bcinr_powl::scheduler::PriorityCapacitySelector {
            capacity: allocation.capacity,
            priority: allocation.priority_map.clone(),
        };
        let execution = self.execute_with_selector(&mut seal_selector, &mut verify_selector)?;

        let allocation_receipt = CmcaAllocationReceipt::from(&allocation);
        let execution_digest = digest_powl_execution_receipt(&execution.powl_receipt);
        let state_digest = digest_state_receipt(&execution.state_receipt);
        let root = root_digest(&allocation_receipt, execution_digest, state_digest);

        let receipt = CmcaPddlPowlExecutionReceipt {
            allocation: allocation_receipt,
            execution: execution.powl_receipt.clone(),
            state: execution.state_receipt.clone(),
            root,
        };

        Ok(CmcaPddlPowlExecution {
            allocation,
            receipt,
            execution,
        })
    }
}

/// Independently verify a [`CmcaPddlPowlExecutionReceipt`] by recomputing
/// the entire allocation from `plan`/`request` -- never from a
/// caller-supplied selector, and never by comparing only the fired trace.
pub fn verify_cmca_execution(
    receipt: &CmcaPddlPowlExecutionReceipt,
    plan: &PddlPowlPlan,
    request: &CmcaExecutionRequest,
) -> Result<(), CmcaAllocationRefusal> {
    let recomputed = allocate_pddl_powl_plan(plan, request)?;

    if recomputed.process_digest != receipt.allocation.process_digest {
        return Err(CmcaAllocationRefusal::ProcessDigestMismatch);
    }
    if recomputed.profile_digest != receipt.allocation.profile_digest {
        return Err(CmcaAllocationRefusal::ProfileDigestMismatch);
    }
    if recomputed.priority_digest != receipt.allocation.priority_digest {
        return Err(CmcaAllocationRefusal::PriorityDigestMismatch);
    }
    if recomputed.mass_field_digest != receipt.allocation.mass_field_digest
        || recomputed.allocation_trace_digest != receipt.allocation.allocation_trace_digest
        || recomputed.tape_root != receipt.allocation.tape_root
        || recomputed.capacity != receipt.allocation.capacity
    {
        return Err(CmcaAllocationRefusal::ProcessDigestMismatch);
    }

    let mut seal_selector = bcinr_powl::scheduler::PriorityCapacitySelector {
        capacity: recomputed.capacity,
        priority: recomputed.priority_map.clone(),
    };
    let replay = bcinr_powl_receipt::execution_v2::execute_and_seal_v2_with_selector(
        &plan.compiled.tape,
        &mut seal_selector,
        &plan.compiled.guards,
        // `plan` is `&PddlPowlPlan` here, whose `max_execution_ticks` is
        // private to `production.rs`; reuse the receipt's own tick count
        // as the bound, since replay only needs to reach the same
        // completion this receipt already reached.
        receipt.execution.tick_count.max(1),
    )
    .map_err(|_| CmcaAllocationRefusal::PriorityDigestMismatch)?;

    if replay.fired_masks != receipt.execution.fired_masks
        || replay.final_done_mask != receipt.execution.final_done_mask
    {
        return Err(CmcaAllocationRefusal::PriorityDigestMismatch);
    }

    Ok(())
}
