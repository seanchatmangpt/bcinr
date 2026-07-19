//! POWL v2 → POWL64 compiler and execution engine reference.
//!
//! `#![allow(dead_code)]`: see `reference/petri.rs`'s module doc comment —
//! same reasoning applies here (a comprehensive reference surface compiled
//! independently into several test binaries, each exercising a different
//! subset).
#![allow(
    dead_code,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::too_many_arguments,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::match_same_arms,
    clippy::unwrap_used,
    clippy::type_complexity,
    clippy::manual_range_contains,
    clippy::string_extend_chars,
    clippy::trivially_copy_pass_by_ref,
    clippy::get_first,
    clippy::recursive_format_impl,
    clippy::large_stack_arrays,
    clippy::to_string_in_format_args,
    clippy::large_enum_variant,
    clippy::boxed_local,
    clippy::unnecessary_wraps
)]

use std::fmt::Write;

// ── AST & Core Types ────────────────────────────────────────────────────────

pub const MAX_NODES: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldLane {
    Law,
    Prereq,
    Custom(u8),
}

impl FieldLane {
    pub fn index(&self) -> usize {
        match self {
            Self::Law => 0,
            Self::Prereq => 1,
            Self::Custom(n) => *n as usize,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobeCell {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl GlobeCell {
    pub fn new(x: u32, y: u32, z: u32) -> Self {
        Self { x, y, z }
    }
    pub fn linear_index(&self) -> usize {
        (self.x + self.y * 100 + self.z * 10000) as usize
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intensity {
    Feel,
    Run,
    Force,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BinaryRelation {
    pub words: [u64; MAX_NODES],
}

impl Default for BinaryRelation {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryRelation {
    pub const fn new() -> Self {
        Self { words: [0u64; MAX_NODES] }
    }
    pub const fn add_edge(&mut self, src: usize, tgt: usize) {
        if src < MAX_NODES && tgt < MAX_NODES {
            self.words[src] |= 1u64 << tgt;
        }
    }
    pub const fn is_edge(&self, src: usize, tgt: usize) -> bool {
        if src < MAX_NODES && tgt < MAX_NODES {
            (self.words[src] >> tgt) & 1 == 1
        } else {
            false
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeTag {
    Leaf,
    Seq,
    Par,
    Choice,
    PartialOrder,
    ChoiceGraph,
    TierShift,
    Mount,
    Loop,
    OpOR,
    OpInterleaving,
    FreqTransition,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug)]
pub enum Powl8Node {
    Silent = 0,
    Activity(FieldLane, Intensity) = 1,
    PartialOrder(u16, u16, BinaryRelation) = 2,
    ChoiceGraph(u16, u16, BinaryRelation, u64, u64) = 3,
    OperatorXOR(u16, u16) = 4,
    OperatorLoop(u16) = 5,
    OperatorSequence(u16, u16) = 6,
    OperatorParallel(u16, u16) = 7,
    OperatorOR(u16, u16) = 8,
    OperatorInterleaving(u16, u16) = 9,
    FrequentTransition(u16, u16) = 10,
    StartNode = 11,
    EndNode = 12,
    Promote(u16) = 13,
    Mounted(FieldLane, u16) = 14,
    Watchdog = 15,
}

impl Powl8Node {
    pub const fn shape(&self) -> ShapeTag {
        match self {
            Self::Silent
            | Self::Activity(..)
            | Self::StartNode
            | Self::EndNode
            | Self::Watchdog => ShapeTag::Leaf,
            Self::OperatorSequence(..) => ShapeTag::Seq,
            Self::OperatorParallel(..) => ShapeTag::Par,
            Self::OperatorXOR(..) => ShapeTag::Choice,
            Self::PartialOrder(..) => ShapeTag::PartialOrder,
            Self::ChoiceGraph(..) => ShapeTag::ChoiceGraph,
            Self::OperatorLoop(..) => ShapeTag::Loop,
            Self::OperatorOR(..) => ShapeTag::OpOR,
            Self::OperatorInterleaving(..) => ShapeTag::OpInterleaving,
            Self::FrequentTransition(..) => ShapeTag::FreqTransition,
            Self::Promote(..) => ShapeTag::TierShift,
            Self::Mounted(..) => ShapeTag::Mount,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Powl8 {
    pub nodes: [Powl8Node; MAX_NODES],
    pub len: usize,
    pub root: u16,
    pub children: [u16; MAX_NODES],
    pub child_len: usize,
}

impl Default for Powl8 {
    fn default() -> Self {
        Self::new()
    }
}

impl Powl8 {
    pub const fn new() -> Self {
        Self {
            nodes: [Powl8Node::Silent; MAX_NODES],
            len: 0,
            root: 0,
            children: [0u16; MAX_NODES],
            child_len: 0,
        }
    }
    pub fn push(&mut self, node: Powl8Node) -> Option<u16> {
        if self.len >= MAX_NODES {
            return None;
        }
        let idx = self.len as u16;
        self.nodes[self.len] = node;
        self.len += 1;
        Some(idx)
    }
    pub fn push_children(&mut self, children: &[u16]) -> Option<u16> {
        if self.child_len + children.len() > MAX_NODES {
            return None;
        }
        let start = self.child_len as u16;
        for &child in children {
            self.children[self.child_len] = child;
            self.child_len += 1;
        }
        Some(start)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidenceMove {
    Warmer,
    Cooler,
}

#[derive(Clone, Copy, Debug)]
pub enum Powl64Node {
    Place(GlobeCell),
    Geodesic(u16, u16),
    Concur(u16, u16),
    Fork(u16, u16),
    PartialOrder(u16, u16, BinaryRelation),
    ChoiceGraph(u16, u16, BinaryRelation, u64, u64),
    Residence(ResidenceMove, u16),
    Descend(u16),
    Cycle(u16),
    Junction(u16, u16),
    Weave(u16, u16),
    FreqArc(u16, u16),
}

impl Powl64Node {
    pub const fn shape(&self) -> ShapeTag {
        match self {
            Self::Place(..) => ShapeTag::Leaf,
            Self::Geodesic(..) => ShapeTag::Seq,
            Self::Concur(..) => ShapeTag::Par,
            Self::Fork(..) => ShapeTag::Choice,
            Self::PartialOrder(..) => ShapeTag::PartialOrder,
            Self::ChoiceGraph(..) => ShapeTag::ChoiceGraph,
            Self::Residence(..) => ShapeTag::TierShift,
            Self::Descend(..) => ShapeTag::Mount,
            Self::Cycle(..) => ShapeTag::Loop,
            Self::Junction(..) => ShapeTag::OpOR,
            Self::Weave(..) => ShapeTag::OpInterleaving,
            Self::FreqArc(..) => ShapeTag::FreqTransition,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Powl64 {
    pub nodes: [Powl64Node; MAX_NODES],
    pub len: usize,
    pub root: u16,
    pub children: [u16; MAX_NODES],
    pub child_len: usize,
}

impl Default for Powl64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Powl64 {
    pub const fn new() -> Self {
        Self {
            nodes: [Powl64Node::Place(GlobeCell { x: 0, y: 0, z: 0 }); MAX_NODES],
            len: 0,
            root: 0,
            children: [0u16; MAX_NODES],
            child_len: 0,
        }
    }
    pub fn push(&mut self, node: Powl64Node) -> Option<u16> {
        if self.len >= MAX_NODES {
            return None;
        }
        let idx = self.len as u16;
        self.nodes[self.len] = node;
        self.len += 1;
        Some(idx)
    }
    pub fn push_children(&mut self, children: &[u16]) -> Option<u16> {
        if self.child_len + children.len() > MAX_NODES {
            return None;
        }
        let start = self.child_len as u16;
        for &child in children {
            self.children[self.child_len] = child;
            self.child_len += 1;
        }
        Some(start)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Motion {
    pub kinetic: Powl8,
    pub geometric: Powl64,
}

impl Motion {
    pub fn new(kinetic: Powl8, geometric: Powl64) -> Result<Self, &'static str> {
        Ok(Self { kinetic, geometric })
    }
}

// ── Compiled Op & Program ──────────────────────────────────────────────────

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Powl64OpKind {
    Activity = 0,
    PartialOrderGate = 1,
    ChoiceGate = 2,
    LoopGate = 3,
    EnterScope = 4,
    ExitScope = 5,
    Promote = 6,
    Demote = 7,
    Watchdog = 8,
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Powl64Op {
    pub kind: Powl64OpKind,
    pub lane: u8,
    pub activity: u16,
    pub scope: u16,
    pub branch: u16,
    pub loop_id: u16,
    pub pred_mask: u64,
    pub succ_mask: u64,
    pub ctrl_mask: u64,
    pub intensity: u8,
    _pad: [u8; 7],
}

impl Powl64Op {
    pub const fn watchdog(scope: u16) -> Self {
        Self {
            kind: Powl64OpKind::Watchdog,
            lane: 0,
            activity: 0,
            scope,
            branch: 0,
            loop_id: 0,
            pred_mask: 0,
            succ_mask: 0,
            ctrl_mask: 0,
            intensity: 0,
            _pad: [0u8; 7],
        }
    }
}

#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct ScopeDesc {
    pub scope_id: u16,
    pub parent: u16,
    pub first_op: u32,
    pub op_count: u32,
    pub tier_hint: u8,
    _pad: [u8; 3],
    pub start_bit: u32,
    pub end_bit: u32,
    pub local_sig: u64,
}

impl ScopeDesc {
    pub const ROOT_PARENT: u16 = 0xFFFF;
    pub const ROOT_END_BIT: u32 = 262_144;
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct UCausalReceipt(pub [u8; 32]);

impl UCausalReceipt {
    pub fn genesis() -> Self {
        Self([0; 32])
    }
    pub fn to_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for &b in &self.0 {
            let _ = write!(s, "{b:02x}");
        }
        s
    }
}

#[derive(Clone, Debug, Default)]
pub struct Powl64Program {
    pub ops: Vec<Powl64Op>,
    pub scopes: Vec<ScopeDesc>,
    pub activity_count: u16,
    pub branch_count: u16,
    pub loop_count: u16,
    pub ucausal_receipt_hash: UCausalReceipt,
}

impl Powl64Program {
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
    pub fn ops_for(&self, scope_id: u16) -> &[Powl64Op] {
        let Some(d) = self.scopes.iter().find(|s| s.scope_id == scope_id) else {
            return &[];
        };
        let start = d.first_op as usize;
        let end = start + d.op_count as usize;
        &self.ops[start..end.min(self.ops.len())]
    }

    fn alloc_scope(&mut self, parent: u16, start_bit: u32, end_bit: u32) -> u16 {
        let id = self.scopes.len() as u16;
        self.scopes.push(ScopeDesc {
            scope_id: id,
            parent,
            first_op: self.ops.len() as u32,
            op_count: 0,
            tier_hint: 0,
            _pad: [0u8; 3],
            start_bit,
            end_bit,
            local_sig: 0,
        });
        id
    }

    fn push(&mut self, op: Powl64Op) {
        self.ops.push(op);
    }
}

// ── Causal Mix mock ────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, Default)]
pub struct UDelta;

pub fn causal_mix(
    r: UCausalReceipt,
    val: u64,
    scope: u16,
    denial: u64,
    _delta: &UDelta,
) -> UCausalReceipt {
    let mut bytes = r.0;
    let mix_val = val ^ (u64::from(scope) << 16) ^ (denial << 32) ^ 0x9e37_79b9_7f4a_7c15;
    let mix_bytes = mix_val.to_ne_bytes();
    for i in 0..8 {
        bytes[i] = bytes[i].wrapping_add(mix_bytes[i]);
        bytes[i + 8] ^= mix_bytes[i];
        bytes[i + 16] = bytes[i + 16].wrapping_sub(mix_bytes[i]);
        bytes[i + 24] ^= mix_bytes[7 - i];
    }
    UCausalReceipt(bytes)
}

fn compute_causal_receipt(ops: &[Powl64Op]) -> UCausalReceipt {
    let mut r = UCausalReceipt::genesis();
    let delta = UDelta;
    for op in ops {
        if op.kind == Powl64OpKind::Activity {
            r = causal_mix(r, u64::from(op.activity), op.scope, op.succ_mask, &delta);
        }
    }
    r
}

// ── Lowering Compiler ──────────────────────────────────────────────────────

pub fn compile(motion: &Motion) -> Powl64Program {
    let mut prog = Powl64Program::default();
    let root = prog.alloc_scope(ScopeDesc::ROOT_PARENT, 0, ScopeDesc::ROOT_END_BIT);
    lower(&mut prog, motion, motion.kinetic.root, motion.geometric.root, root, 0);
    recompute_scope_ranges(&mut prog);
    prog.ucausal_receipt_hash = compute_causal_receipt(&prog.ops);
    prog
}

fn fold_succ(ops: &[Powl64Op]) -> u64 {
    let mut acc = 0u64;
    for op in ops {
        acc |= op.succ_mask;
    }
    acc
}

fn recompute_scope_ranges(prog: &mut Powl64Program) {
    for s in &mut prog.scopes {
        let mut first: Option<u32> = None;
        let mut count: u32 = 0;
        for (i, op) in prog.ops.iter().enumerate() {
            if op.scope == s.scope_id {
                if first.is_none() {
                    first = Some(i as u32);
                }
                count += 1;
            }
        }
        s.first_op = first.unwrap_or(0);
        s.op_count = count;
    }
}

fn lower(prog: &mut Powl64Program, motion: &Motion, k_idx: u16, g_idx: u16, scope: u16, depth: u8) {
    let kn = &motion.kinetic.nodes[k_idx as usize];
    let gn = &motion.geometric.nodes[g_idx as usize];

    match (kn, gn) {
        (Powl8Node::Activity(lane, intensity), Powl64Node::Place(cell)) => {
            let activity = prog.activity_count;
            prog.activity_count = prog.activity_count.wrapping_add(1);
            let ctrl = (cell.linear_index() as u64) & 0x0003_FFFF;
            prog.push(Powl64Op {
                kind: Powl64OpKind::Activity,
                lane: lane.index() as u8,
                activity,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 1u64 << (activity & 63),
                ctrl_mask: ctrl,
                intensity: match intensity {
                    Intensity::Feel => 0,
                    Intensity::Run => 1,
                    Intensity::Force => 2,
                },
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::OperatorSequence(ka, kb), Powl64Node::Geodesic(ga, gb)) => {
            let before = prog.ops.len();
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            let mid = prog.ops.len();
            lower(prog, motion, *kb, *gb, scope, depth + 1);
            let pred = fold_succ(&prog.ops[before..mid]);
            let succ = fold_succ(&prog.ops[mid..]);
            prog.push(Powl64Op {
                kind: Powl64OpKind::PartialOrderGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: pred,
                succ_mask: succ,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::OperatorParallel(ka, kb), Powl64Node::Concur(ga, gb)) => {
            let before = prog.ops.len();
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            let mid = prog.ops.len();
            lower(prog, motion, *kb, *gb, scope, depth + 1);
            let pred = fold_succ(&prog.ops[before..mid]) | fold_succ(&prog.ops[mid..]);
            prog.push(Powl64Op {
                kind: Powl64OpKind::PartialOrderGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: pred,
                succ_mask: 0,
                ctrl_mask: u64::MAX, // concur marker
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::OperatorXOR(ka, kb), Powl64Node::Fork(ga, gb)) => {
            let branch_a = prog.branch_count;
            let branch_b = prog.branch_count.wrapping_add(1);
            prog.branch_count = prog.branch_count.wrapping_add(2);
            prog.push(Powl64Op {
                kind: Powl64OpKind::ChoiceGate,
                lane: 0,
                activity: 0,
                scope,
                branch: branch_a,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: (1u64 << (branch_a & 63)) | (1u64 << (branch_b & 63)),
                intensity: 0,
                _pad: [0u8; 7],
            });
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            lower(prog, motion, *kb, *gb, scope, depth + 1);
        }

        (Powl8Node::OperatorLoop(ki), Powl64Node::Cycle(gi)) => {
            let lid = prog.loop_count;
            prog.loop_count = prog.loop_count.wrapping_add(1);
            prog.push(Powl64Op {
                kind: Powl64OpKind::LoopGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: lid,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: 1u64 << (lid & 63),
                intensity: 0,
                _pad: [0u8; 7],
            });
            let body_start = prog.ops.len();
            lower(prog, motion, *ki, *gi, scope, depth + 1);
            let pred = fold_succ(&prog.ops[body_start..]);
            prog.push(Powl64Op {
                kind: Powl64OpKind::LoopGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: lid,
                pred_mask: pred,
                succ_mask: 0,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::Watchdog, Powl64Node::Place(_)) => {
            prog.push(Powl64Op::watchdog(scope));
        }

        (Powl8Node::Promote(ki), Powl64Node::Residence(move_, gi)) => {
            let kind = match move_ {
                ResidenceMove::Warmer => Powl64OpKind::Promote,
                ResidenceMove::Cooler => Powl64OpKind::Demote,
            };
            prog.push(Powl64Op {
                kind,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
            lower(prog, motion, *ki, *gi, scope, depth + 1);
        }

        (Powl8Node::Mounted(_, ki), Powl64Node::Descend(gi)) => {
            let child_scope = prog.alloc_scope(scope, 0, ScopeDesc::ROOT_END_BIT);
            prog.push(Powl64Op {
                kind: Powl64OpKind::EnterScope,
                lane: 0,
                activity: 0,
                scope: child_scope,
                branch: 0,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
            lower(prog, motion, *ki, *gi, child_scope, depth + 1);
            prog.push(Powl64Op {
                kind: Powl64OpKind::ExitScope,
                lane: 0,
                activity: 0,
                scope: child_scope,
                branch: 0,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::OperatorOR(ka, kb), Powl64Node::Junction(ga, gb)) => {
            let branch_a = prog.branch_count;
            let branch_b = prog.branch_count.wrapping_add(1);
            prog.branch_count = prog.branch_count.wrapping_add(2);
            let inclusive_ctrl = (1u64 << (branch_a & 63)) | (1u64 << (branch_b & 63));
            prog.push(Powl64Op {
                kind: Powl64OpKind::ChoiceGate,
                lane: 0,
                activity: 0,
                scope,
                branch: branch_a,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: inclusive_ctrl,
                intensity: 0,
                _pad: [0u8; 7],
            });
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            prog.push(Powl64Op {
                kind: Powl64OpKind::ChoiceGate,
                lane: 0,
                activity: 0,
                scope,
                branch: branch_b,
                loop_id: 0,
                pred_mask: 0,
                succ_mask: 0,
                ctrl_mask: inclusive_ctrl,
                intensity: 0,
                _pad: [0u8; 7],
            });
            lower(prog, motion, *kb, *gb, scope, depth + 1);
        }

        (Powl8Node::OperatorInterleaving(ka, kb), Powl64Node::Weave(ga, gb)) => {
            let before = prog.ops.len();
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            let mid = prog.ops.len();
            lower(prog, motion, *kb, *gb, scope, depth + 1);
            let pred_a = fold_succ(&prog.ops[before..mid]);
            let pred_b = fold_succ(&prog.ops[mid..]);
            prog.push(Powl64Op {
                kind: Powl64OpKind::PartialOrderGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: pred_a | pred_b,
                succ_mask: 0,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        (Powl8Node::FrequentTransition(ka, kb), Powl64Node::FreqArc(ga, gb)) => {
            let before = prog.ops.len();
            lower(prog, motion, *ka, *ga, scope, depth + 1);
            let mid = prog.ops.len();
            lower(prog, motion, *kb, *gb, scope, depth + 1);
            let pred = fold_succ(&prog.ops[before..mid]);
            let succ = fold_succ(&prog.ops[mid..]);
            prog.push(Powl64Op {
                kind: Powl64OpKind::PartialOrderGate,
                lane: 0,
                activity: 0,
                scope,
                branch: 0,
                loop_id: 0,
                pred_mask: pred,
                succ_mask: succ,
                ctrl_mask: 0,
                intensity: 0,
                _pad: [0u8; 7],
            });
        }

        _ => {}
    }
}

// ── Execution Engine (Powl64Executor) ──────────────────────────────────────

pub const MAX_SCOPES: usize = 64;
pub const MAX_SCOPE_DEPTH: usize = 16;
pub const WATCHDOG_DRAIN_DENIAL: u64 = 0xFFFF_FFFF_FFFF_FFFF;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidenceTier {
    Reg = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    Dram = 4,
    Nvm = 5,
}

#[derive(Clone, Debug)]
pub struct ResidenceMap<const N: usize> {
    pub tiers: [ResidenceTier; N],
}

impl<const N: usize> ResidenceMap<N> {
    pub fn from_program(_prog: &Powl64Program) -> Self {
        Self { tiers: [ResidenceTier::Reg; N] }
    }
    pub fn tier(&self, scope: u16) -> ResidenceTier {
        self.tiers[scope as usize]
    }
    pub fn set_tier(&mut self, scope: u16, tier: ResidenceTier) {
        self.tiers[scope as usize] = tier;
    }
}

#[derive(Clone, Debug)]
pub struct Watchdog {
    deadline: u64,
    tripped: std::cell::Cell<bool>,
}

impl Watchdog {
    pub fn with_deadline(deadline: u64) -> Self {
        Self { deadline, tripped: std::cell::Cell::new(false) }
    }
    pub fn cycle(&self) -> bool {
        self.tripped.set(true);
        true
    }
    pub fn is_tripped(&self) -> bool {
        self.tripped.get()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SlotId(pub u8);

impl SlotId {
    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

pub struct Slot {
    pub active: std::cell::Cell<bool>,
}

impl Slot {
    pub fn release(&self) {
        self.active.set(false);
    }
}

pub struct Dispatcher {
    pub slots: [Slot; 64],
}

impl Dispatcher {
    pub fn new() -> Self {
        Self { slots: std::array::from_fn(|_| Slot { active: std::cell::Cell::new(false) }) }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowedProjection {
    start_bit: u32,
    end_bit: u32,
}

impl WindowedProjection {
    pub fn root() -> Self {
        Self { start_bit: 0, end_bit: ScopeDesc::ROOT_END_BIT }
    }
    pub fn start_bit(&self) -> u32 {
        self.start_bit
    }
    pub fn end_bit(&self) -> u32 {
        self.end_bit
    }
}

pub fn enter_scope(
    parent: &WindowedProjection,
    child: &ScopeDesc,
) -> Result<WindowedProjection, &'static str> {
    if child.start_bit < parent.start_bit || child.end_bit > parent.end_bit {
        return Err("Child window is not a subset of parent window");
    }
    Ok(WindowedProjection { start_bit: child.start_bit, end_bit: child.end_bit })
}

pub enum WatchdogOutcome {
    Continue,
    Drain(Vec<DrainFragment>),
}

pub struct DrainFragment {
    pub op_index: u32,
    pub denial: u64,
}

pub fn drive_watchdog(
    _op: &Powl64Op,
    op_index: u32,
    scope_remaining: &[Powl64Op],
    watchdog: &Watchdog,
) -> WatchdogOutcome {
    if watchdog.is_tripped() {
        let mut frags = Vec::new();
        for (idx, _drained_op) in scope_remaining.iter().enumerate() {
            frags.push(DrainFragment {
                op_index: (op_index + 1 + idx as u32),
                denial: WATCHDOG_DRAIN_DENIAL,
            });
        }
        WatchdogOutcome::Drain(frags)
    } else {
        WatchdogOutcome::Continue
    }
}

pub enum ConcurOutcome {
    Parallel { slots: [SlotId; 2] },
    Sequential,
}

pub fn schedule_concur(
    _left: u32,
    _right: u32,
    _dispatcher: &Dispatcher,
    _tier: u8,
) -> ConcurOutcome {
    ConcurOutcome::Parallel { slots: [SlotId(0), SlotId(1)] }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExecutionEvent {
    pub op_index: u32,
    pub kind: Powl64OpKind,
    pub scope: u16,
    pub denial: u64,
    pub drained: bool,
    pub concur_slots: Option<[u8; 2]>,
}

#[derive(Clone, Debug)]
pub struct ExecutionReport {
    pub events: Vec<ExecutionEvent>,
    pub residence: ResidenceMap<MAX_SCOPES>,
    pub scopes_entered: u32,
    pub concur_slot_claims: u32,
}

impl ExecutionReport {
    pub fn receipt(&self) -> UCausalReceipt {
        let mut r = UCausalReceipt::genesis();
        let zero = UDelta;
        for ev in &self.events {
            r = causal_mix(r, u64::from(ev.op_index), ev.scope, ev.denial, &zero);
        }
        r
    }
    pub fn canonical_receipt_hash(&self) -> String {
        format!("blake3:{}", self.receipt().to_hex())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutorError {
    ScopeOverflow { scope_id: u16 },
    ScopeDepthOverflow,
    UnbalancedExit { op_index: u32 },
    UnknownScope { scope_id: u16 },
    ScopeWindow(&'static str),
    ConcurWithoutLeftArm { op_index: u32 },
}

pub struct Powl64Executor<'p> {
    program: &'p Powl64Program,
}

impl<'p> Powl64Executor<'p> {
    pub fn new(program: &'p Powl64Program) -> Self {
        Self { program }
    }

    pub fn run(
        &self,
        watchdog: &Watchdog,
        dispatcher: &Dispatcher,
    ) -> Result<ExecutionReport, ExecutorError> {
        let ops = &self.program.ops;
        let scopes = &self.program.scopes;

        // Validates scope 0 exists (`?` propagates `UnknownScope` if not);
        // the descriptor itself isn't otherwise read below.
        let _root_desc = scopes
            .iter()
            .find(|s| s.scope_id == 0)
            .ok_or(ExecutorError::UnknownScope { scope_id: 0 })?;
        let root_proj = WindowedProjection::root();

        let mut scope_stack = Vec::with_capacity(MAX_SCOPE_DEPTH);
        scope_stack.push((0u16, root_proj));

        let mut residence = ResidenceMap::from_program(self.program);
        let mut events = Vec::with_capacity(ops.len());
        let mut scopes_entered = 0u32;
        let mut concur_slot_claims = 0u32;

        let mut i = 0usize;
        while i < ops.len() {
            let op = &ops[i];

            if !scopes.iter().any(|s| s.scope_id == op.scope) {
                return Err(ExecutorError::UnknownScope { scope_id: op.scope });
            }

            match op.kind {
                Powl64OpKind::EnterScope => {
                    if scope_stack.len() >= MAX_SCOPE_DEPTH {
                        return Err(ExecutorError::ScopeDepthOverflow);
                    }
                    if op.scope as usize >= MAX_SCOPES {
                        return Err(ExecutorError::ScopeOverflow { scope_id: op.scope });
                    }
                    let child_desc = scopes
                        .iter()
                        .find(|s| s.scope_id == op.scope)
                        .ok_or(ExecutorError::UnknownScope { scope_id: op.scope })?;
                    let parent = &scope_stack
                        .last()
                        .ok_or(ExecutorError::UnbalancedExit { op_index: i as u32 })?
                        .1;
                    let child_proj =
                        enter_scope(parent, child_desc).map_err(ExecutorError::ScopeWindow)?;
                    scope_stack.push((op.scope, child_proj));
                    scopes_entered = scopes_entered.saturating_add(1);
                    push_admit(&mut events, op, i);
                    i += 1;
                }

                Powl64OpKind::ExitScope => {
                    if scope_stack.len() < 2 {
                        return Err(ExecutorError::UnbalancedExit { op_index: i as u32 });
                    }
                    scope_stack.pop();
                    push_admit(&mut events, op, i);
                    i += 1;
                }

                Powl64OpKind::Promote => {
                    residence.set_tier(op.scope, ResidenceTier::L1);
                    push_admit(&mut events, op, i);
                    i += 1;
                }

                Powl64OpKind::Demote => {
                    residence.set_tier(op.scope, ResidenceTier::Dram);
                    push_admit(&mut events, op, i);
                    i += 1;
                }

                Powl64OpKind::Watchdog => {
                    let drain_end = scope_run_end(ops, i, op.scope);
                    let scope_remaining = &ops[i + 1..drain_end];
                    let outcome = drive_watchdog(op, i as u32, scope_remaining, watchdog);
                    push_admit(&mut events, op, i);
                    match outcome {
                        WatchdogOutcome::Continue => {
                            i += 1;
                        }
                        WatchdogOutcome::Drain(drain) => {
                            for frag in drain {
                                let drained_op = &ops[frag.op_index as usize];
                                events.push(ExecutionEvent {
                                    op_index: frag.op_index,
                                    kind: drained_op.kind,
                                    scope: drained_op.scope,
                                    denial: frag.denial,
                                    drained: true,
                                    concur_slots: None,
                                });
                            }
                            i = drain_end;
                        }
                    }
                }

                Powl64OpKind::PartialOrderGate if op.ctrl_mask == u64::MAX => {
                    if i < 1 {
                        return Err(ExecutorError::ConcurWithoutLeftArm { op_index: i as u32 });
                    }
                    let left_idx = i.saturating_sub(2) as u32;
                    let right_idx = i.saturating_sub(1) as u32;
                    let outcome = schedule_concur(
                        left_idx,
                        right_idx,
                        dispatcher,
                        residence.tier(op.scope) as u8,
                    );
                    let slot_pair = match outcome {
                        ConcurOutcome::Parallel { slots } => {
                            concur_slot_claims = concur_slot_claims.saturating_add(2);
                            Some([slots[0].0, slots[1].0])
                        }
                        ConcurOutcome::Sequential => None,
                    };
                    events.push(ExecutionEvent {
                        op_index: i as u32,
                        kind: op.kind,
                        scope: op.scope,
                        denial: 0,
                        drained: false,
                        concur_slots: slot_pair,
                    });
                    i += 1;
                }

                _ => {
                    push_admit(&mut events, op, i);
                    i += 1;
                }
            }
        }

        Ok(ExecutionReport { events, residence, scopes_entered, concur_slot_claims })
    }
}

fn push_admit(events: &mut Vec<ExecutionEvent>, op: &Powl64Op, idx: usize) {
    events.push(ExecutionEvent {
        op_index: idx as u32,
        kind: op.kind,
        scope: op.scope,
        denial: 0,
        drained: false,
        concur_slots: None,
    });
}

fn scope_run_end(ops: &[Powl64Op], start: usize, scope_id: u16) -> usize {
    let mut j = start + 1;
    while j < ops.len() && ops[j].scope == scope_id {
        j += 1;
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_powl_compile_execute() {
        let mut k = Powl8::new();
        let mut g = Powl64::new();
        let k_act = k.push(Powl8Node::Activity(FieldLane::Law, Intensity::Run)).unwrap();
        let g_place = g.push(Powl64Node::Place(GlobeCell::new(0, 0, 0))).unwrap();
        k.root = k_act;
        g.root = g_place;

        let motion = Motion::new(k, g).unwrap();
        let program = compile(&motion);
        assert_eq!(program.activity_count, 1);

        let watchdog = Watchdog::with_deadline(100);
        let dispatcher = Dispatcher::new();
        let executor = Powl64Executor::new(&program);
        let report = executor.run(&watchdog, &dispatcher).unwrap();
        assert_eq!(report.events.len(), 1);
        assert_eq!(report.events[0].kind, Powl64OpKind::Activity);
    }
}
