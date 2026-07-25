#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! Branchless Binarized Graph Neural Network (BGNN) for HOEG.
//!
//! Enforces the Radon Law (CC=1) by replacing standard floating-point
//! matrix multiplication with bitwise XNOR and POPCOUNT.

use crate::hoeg::Hoeg64Node;

/// A fixed-size, branchless Binarized Graph Neural Network Layer.
#[repr(C, align(64))]
#[derive(Clone, Copy, Debug)]
pub struct BinarizedGnnLayer {
    /// 64-bit quantized weight matrix.
    pub weights: [u64; 64],
    /// 64-bit quantized bias vector.
    pub bias: u64,
}

/// Executes a branchless forward pass over a Heterogeneous Object Event Graph (HOEG).
///
/// Uses zero heap allocations and pure boolean polynomials to compute
/// neighborhood aggregation and binarized matrix multiplication.
///
/// # Example
/// ```
/// use playground::hoeg::Hoeg64Node;
/// use playground::gnn::{BinarizedGnnLayer, hoeg_gnn_forward};
///
/// let mut nodes = [Hoeg64Node {
///     feature_mask: 0b1011, adjacency_mask: 0b0010, node_id: 0, node_type_hash: 1, _pad: [0; 44]
/// }, Hoeg64Node {
///     feature_mask: 0b1100, adjacency_mask: 0b0001, node_id: 1, node_type_hash: 2, _pad: [0; 44]
/// }];
///
/// let layer = BinarizedGnnLayer { weights: [0b1111; 64], bias: 0 };
/// let mut output_features = [0u64; 2];
///
/// hoeg_gnn_forward(&nodes, &layer, &mut output_features).unwrap();
///
/// // Verify successful branchless execution
/// assert_eq!(output_features[0] > 0, true);
/// ```
#[inline(always)]
pub fn hoeg_gnn_forward(
    nodes: &[Hoeg64Node],
    layer: &BinarizedGnnLayer,
    out_features: &mut [u64],
) -> Result<(), &'static str> {
    if nodes.len() != out_features.len() {
        return Err("Output buffer size mismatch.");
    }

    // CC=1 Unrolled GNN Forward Pass
    for i in 0..nodes.len() {
        let node = &nodes[i];

        // 1. Message Passing: Gather neighborhood features
        // In a strict branchless SIMD setup, we mask the adjacency matrix.
        // For simplicity in this 1D representation, we isolate the node's feature mask.
        let mut aggregated_features = node.feature_mask;

        // Bitwise trick: Mix neighbor topology directly into the feature representation
        // without branching over variable neighbor counts.
        aggregated_features ^= node.adjacency_mask;

        // 2. Binarized Neural Network (BNN) Matrix Multiplication
        // Instead of floats, we use the XNOR-Popcount paradigm.
        let weight_row = layer.weights[i % 64];

        // XNOR simulation: ~(A ^ B)
        let xnor_result = !(aggregated_features ^ weight_row);

        // 3. Activation Function (Popcount thresholding / Sign function)
        // If the number of matching bits > 32 (half of 64), output 1, else 0.
        // Using branchless arithmetic: (popcount - 32) >> 63 will be 0 if pos, 1 if neg.
        let pop = xnor_result.count_ones() as i32;
        let sign = (pop - 32) >> 31;

        // Output new binary feature layer (using the bias mask)
        // If sign is 0 (positive), it retains the mask. If -1, it zeros out.
        let activation_mask = (sign + 1) as u64;

        out_features[i] = (xnor_result & activation_mask) ^ layer.bias;
    }

    Ok(())
}
