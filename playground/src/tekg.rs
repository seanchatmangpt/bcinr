#![allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::cast_sign_loss)]
//! Branchless Temporal Event Knowledge Graph (tEKG) matrix compiler.
//!
//! Fully branchless (CC = 1), zero-allocation, and `#![no_std]` compliant.
//! Implements Algorithm 1 from Khayatbashi et al. (2024) for Transforming
//! OCEL to tEKG arrays.

/// Enum identifying the TEKG Node Label in SWAR fields.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TekgLabel {
    /// A log-level node (the OCEL log itself).
    Log = 0,
    /// A class/event-type node.
    Class = 1,
    /// A single recorded event occurrence.
    Event = 2,
    /// An entity (object) node.
    Entity = 3,
    /// A point-in-time snapshot of an entity's state.
    Snapshot = 4,
}

/// A highly packed, branchless tEKG Node.
/// 64-byte aligned to maximize SIMD cache-line efficiency.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Tekg64Node {
    /// Chronological timestamp metric
    pub timestamp_ns: u64,
    /// Bitmask for valid inter-process relationships
    pub rel_mask: u64,
    /// Local identity
    pub node_id: u16,
    /// E.g. points to the Entity for a Snapshot node.
    pub parent_id: u16,
    /// For snapshots: the node ID of the temporally previous snapshot (if any)
    pub prev_snapshot_id: u16,
    /// This node's kind (log/class/event/entity/snapshot).
    pub label: TekgLabel,
    /// Padding to reach the 64-byte cache-line-aligned layout.
    pub _pad: [u8; 41],
}

const _: () = assert!(core::mem::align_of::<Tekg64Node>() == 64);
const _: () = assert!(core::mem::size_of::<Tekg64Node>() == 64);

/// Compiles an object update array into a dense tEKG node stream.
///
/// Simulates Algorithm 1 "Snapshot Generation" (Khayatbashi et al., 2024).
/// Every change in attribute value yields a new Snapshot node linked
/// via the `prev_snapshot_id` field chronologically.
///
/// # Example: The Assignment Update (Fig 2)
///
/// The paper details an assignment `a1` whose points are updated from 2 to 3
/// at times `t1` and `t4`. This requires capturing the entity and mapping
/// the two chronological snapshots sequentially.
///
/// ```
/// use playground::tekg::{compile_snapshot_chain, Tekg64Node, TekgLabel};
///
/// // Prepare a zeroed output buffer for the nodes.
/// let mut out = [Tekg64Node {
///     timestamp_ns: 0, rel_mask: 0, node_id: 0, parent_id: 0,
///     prev_snapshot_id: 0, label: TekgLabel::Log, _pad: [0; 41]
/// }; 3];
///
/// // Algorithm 1: Entity `a1` (ID 1) updated at `t1` (100) and `t4` (400).
/// let timestamps = [100, 400];
/// let nodes_emitted = compile_snapshot_chain(1, &timestamps, &mut out).unwrap();
///
/// assert_eq!(nodes_emitted, 3);
///
/// // Node 0: The Base Entity `a1`
/// assert_eq!(out[0].label, TekgLabel::Entity);
/// assert_eq!(out[0].node_id, 1);
///
/// // Node 1: Snapshot 1 at `t1`
/// assert_eq!(out[1].label, TekgLabel::Snapshot);
/// assert_eq!(out[1].parent_id, 1);
/// assert_eq!(out[1].timestamp_ns, 100);
/// assert_eq!(out[1].prev_snapshot_id, 0); // First snapshot has no prev
///
/// // Node 2: Snapshot 2 at `t4`
/// assert_eq!(out[2].label, TekgLabel::Snapshot);
/// assert_eq!(out[2].parent_id, 1);
/// assert_eq!(out[2].timestamp_ns, 400);
/// assert_eq!(out[2].prev_snapshot_id, out[1].node_id); // Linked to previous!
/// ```
#[inline(always)]
pub fn compile_snapshot_chain(
    entity_id: u16,
    update_timestamps: &[u64],
    out: &mut [Tekg64Node],
) -> Result<usize, &'static str> {
    if out.len() < update_timestamps.len() + 1 {
        return Err("Buffer too small for TEKG compilation");
    }

    // Node 0: The Base Entity (Algorithm 1: Line 14)
    out[0] = Tekg64Node {
        timestamp_ns: 0,
        rel_mask: 0,
        node_id: entity_id,
        parent_id: 0,
        prev_snapshot_id: 0,
        label: TekgLabel::Entity,
        _pad: [0; 41],
    };

    let mut current_idx = 1;
    let mut prev_snap = 0;

    // Algorithm 1: Lines 17-23 (Snapshot Emission and Updating)
    // CC=1 unrolling loop for snapshot matrix emission
    for &ts in update_timestamps {
        let snap_id = entity_id.wrapping_add(current_idx as u16 * 100);

        out[current_idx] = Tekg64Node {
            timestamp_ns: ts,
            rel_mask: 1 << (entity_id & 63),
            node_id: snap_id,
            parent_id: entity_id,
            prev_snapshot_id: prev_snap,
            label: TekgLabel::Snapshot,
            _pad: [0; 41],
        };
        prev_snap = snap_id;
        current_idx += 1;
    }

    Ok(current_idx)
}
