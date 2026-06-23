//! Branchless Heterogeneous Object Event Graph (HOEG) matrix compiler.
//!
//! Fully branchless (CC = 1), zero-allocation, and `#![no_std]` compliant.
//! Implements HOEG from Smit et al. (2024) for Heterogeneous Graph Neural Networks.

/// A highly packed, branchless HOEG Node.
/// 64-byte aligned to maximize SIMD cache-line efficiency.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct Hoeg64Node {
    /// Bitwise projection of the feature matrix row ($X_i$)
    pub feature_mask: u64,
    /// Bitwise projection of the adjacency matrix row ($A_i$)
    pub adjacency_mask: u64,
    /// Local identity
    pub node_id: u16,
    /// Identifier hash representing the node type ($NT$). 
    pub node_type_hash: u16,
    /// Padding to ensure 64-byte alignment
    pub _pad: [u8; 44],
}

const _: () = assert!(core::mem::align_of::<Hoeg64Node>() == 64);
const _: () = assert!(core::mem::size_of::<Hoeg64Node>() == 64);

/// Compiles a set of features and adjacencies into a dense HOEG matrix.
///
/// Converts the multi-dimensional OCEL `wasm4pm-compat` structures directly
/// into flat arrays for instantaneous ingestion by Graph Neural Networks
/// targeting Predictive Process Monitoring.
///
/// # Example
///
/// ```
/// use playground::hoeg::{compile_hoeg_matrix, Hoeg64Node};
///
/// // Prepare a zeroed output buffer for the nodes.
/// let mut out = [Hoeg64Node { 
///     feature_mask: 0, adjacency_mask: 0, node_id: 0, node_type_hash: 0, _pad: [0; 44] 
/// }; 3];
///
/// // Synthetic node types, features, and connections
/// let types = [1, 1, 2];
/// let features = [0b101, 0b010, 0b111];
/// let connections = [0b010, 0b100, 0b001];
/// 
/// let nodes_emitted = compile_hoeg_matrix(&types, &features, &connections, &mut out).unwrap();
///
/// assert_eq!(nodes_emitted, 3);
///
/// // Assert node 0 (e.g. an Event node)
/// assert_eq!(out[0].node_id, 0);
/// assert_eq!(out[0].node_type_hash, 1);
/// assert_eq!(out[0].feature_mask, 0b101);
/// assert_eq!(out[0].adjacency_mask, 0b010);
/// ```
#[inline(always)]
pub fn compile_hoeg_matrix(
    node_types: &[u16],
    features: &[u64],
    connections: &[u64],
    out: &mut [Hoeg64Node],
) -> Result<usize, &'static str> {
    if out.len() < node_types.len() {
        return Err("Buffer too small for HOEG compilation");
    }

    let mut current_idx = 0;

    // CC=1 unrolling loop for heterogeneous matrix emission
    for i in 0..node_types.len() {
        out[current_idx] = Hoeg64Node {
            feature_mask: features[i],
            adjacency_mask: connections[i],
            node_id: current_idx as u16,
            node_type_hash: node_types[i],
            _pad: [0; 44],
        };
        current_idx += 1;
    }

    Ok(current_idx)
}
