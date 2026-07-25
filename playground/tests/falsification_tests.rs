#![allow(warnings, clippy::all)]
#![allow(warnings)]
//! Property-based falsification tests for `playground`'s HOEG/TEKG/GNN
//! compilation pipeline — adversarial proptest cases aimed at breaking the
//! compiled-matrix/snapshot-chain invariants rather than confirming them.
use playground::{
    gnn::{hoeg_gnn_forward, BinarizedGnnLayer},
    hoeg::{compile_hoeg_matrix, Hoeg64Node},
    tekg::{compile_snapshot_chain, Tekg64Node, TekgLabel},
};
use proptest::prelude::*;

// Helper for comparisons
fn hoeg_nodes_eq(a: &[Hoeg64Node], b: &[Hoeg64Node]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (na, nb) in a.iter().zip(b.iter()) {
        if na.feature_mask != nb.feature_mask
            || na.adjacency_mask != nb.adjacency_mask
            || na.node_id != nb.node_id
            || na.node_type_hash != nb.node_type_hash
        {
            return false;
        }
    }
    true
}

fn tekg_nodes_eq(a: &[Tekg64Node], b: &[Tekg64Node]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (na, nb) in a.iter().zip(b.iter()) {
        if na.timestamp_ns != nb.timestamp_ns
            || na.rel_mask != nb.rel_mask
            || na.node_id != nb.node_id
            || na.parent_id != nb.parent_id
            || na.prev_snapshot_id != nb.prev_snapshot_id
            || na.label != nb.label
        {
            return false;
        }
    }
    true
}

// --- HOEG Mutants ---
fn mutant1_compile_hoeg_matrix(
    node_types: &[u16],
    features: &[u64],
    connections: &[u64],
    out: &mut [Hoeg64Node],
) -> Result<usize, &'static str> {
    let res = compile_hoeg_matrix(node_types, features, connections, out);
    for i in 0..node_types.len() {
        out[i].feature_mask = 0; // Mutant: constant feature mask
    }
    res
}

fn mutant2_compile_hoeg_matrix(
    node_types: &[u16],
    features: &[u64],
    connections: &[u64],
    out: &mut [Hoeg64Node],
) -> Result<usize, &'static str> {
    let res = compile_hoeg_matrix(node_types, features, connections, out);
    for i in 0..node_types.len() {
        out[i].adjacency_mask = connections[i].wrapping_shr(1); // Mutant: bit-shifted connections
    }
    res
}

fn mutant3_compile_hoeg_matrix(
    node_types: &[u16],
    features: &[u64],
    connections: &[u64],
    out: &mut [Hoeg64Node],
) -> Result<usize, &'static str> {
    let res = compile_hoeg_matrix(node_types, features, connections, out);
    for i in 0..node_types.len() {
        out[i].node_id = out[i].node_id.wrapping_add(1); // Mutant: wrong node id
    }
    res
}

// --- GNN Mutants ---
fn mutant1_hoeg_gnn_forward(
    nodes: &[Hoeg64Node],
    layer: &BinarizedGnnLayer,
    out_features: &mut [u64],
) -> Result<(), &'static str> {
    if nodes.len() != out_features.len() {
        return Err("Output buffer size mismatch.");
    }
    for i in 0..nodes.len() {
        let node = &nodes[i];
        let aggregated_features = node.feature_mask; // Mutant: missing ^ node.adjacency_mask
        let weight_row = layer.weights[i % 64];
        let xnor_result = !(aggregated_features ^ weight_row);
        let pop = xnor_result.count_ones() as i32;
        let sign = (pop - 32) >> 31;
        let activation_mask = (sign + 1) as u64;
        out_features[i] = (xnor_result & activation_mask) ^ layer.bias;
    }
    Ok(())
}

fn mutant2_hoeg_gnn_forward(
    nodes: &[Hoeg64Node],
    layer: &BinarizedGnnLayer,
    out_features: &mut [u64],
) -> Result<(), &'static str> {
    if nodes.len() != out_features.len() {
        return Err("Output buffer size mismatch.");
    }
    for i in 0..nodes.len() {
        let node = &nodes[i];
        let mut aggregated_features = node.feature_mask;
        aggregated_features ^= node.adjacency_mask;
        let weight_row = layer.weights[0]; // Mutant: always weight 0
        let xnor_result = !(aggregated_features ^ weight_row);
        let pop = xnor_result.count_ones() as i32;
        let sign = (pop - 32) >> 31;
        let activation_mask = (sign + 1) as u64;
        out_features[i] = (xnor_result & activation_mask) ^ layer.bias;
    }
    Ok(())
}

fn mutant3_hoeg_gnn_forward(
    nodes: &[Hoeg64Node],
    layer: &BinarizedGnnLayer,
    out_features: &mut [u64],
) -> Result<(), &'static str> {
    if nodes.len() != out_features.len() {
        return Err("Output buffer size mismatch.");
    }
    for i in 0..nodes.len() {
        let node = &nodes[i];
        let mut aggregated_features = node.feature_mask;
        aggregated_features ^= node.adjacency_mask;
        let weight_row = layer.weights[i % 64];
        let xnor_result = !(aggregated_features ^ weight_row);
        let pop = xnor_result.count_ones() as i32;
        let sign = (pop - 32) >> 31;
        let activation_mask = (sign + 1) as u64;
        out_features[i] = (xnor_result & activation_mask) ^ (!layer.bias); // Mutant: flipped bias
    }
    Ok(())
}

// --- TEKG Mutants ---
fn mutant1_compile_snapshot_chain(
    entity_id: u16,
    update_timestamps: &[u64],
    out: &mut [Tekg64Node],
) -> Result<usize, &'static str> {
    let res = compile_snapshot_chain(entity_id, update_timestamps, out);
    if let Ok(_) = res {
        out[0].label = TekgLabel::Log; // Mutant: Wrong base label
    }
    res
}

fn mutant2_compile_snapshot_chain(
    entity_id: u16,
    update_timestamps: &[u64],
    out: &mut [Tekg64Node],
) -> Result<usize, &'static str> {
    let res = compile_snapshot_chain(entity_id, update_timestamps, out);
    if let Ok(count) = res {
        for i in 1..count {
            out[i].rel_mask = 0; // Mutant: Wrong rel mask
        }
    }
    res
}

fn mutant3_compile_snapshot_chain(
    entity_id: u16,
    update_timestamps: &[u64],
    out: &mut [Tekg64Node],
) -> Result<usize, &'static str> {
    let res = compile_snapshot_chain(entity_id, update_timestamps, out);
    if let Ok(count) = res {
        for i in 1..count {
            out[i].prev_snapshot_id = 0; // Mutant: broken chain
        }
    }
    res
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_hoeg_determinism(
        node_types in prop::collection::vec(any::<u16>(), 1..64),
        features in prop::collection::vec(any::<u64>(), 1..64),
        connections in prop::collection::vec(any::<u64>(), 1..64),
    ) {
        let min_len = node_types.len().min(features.len()).min(connections.len());
        let nt = &node_types[..min_len];
        let ft = &features[..min_len];
        let cn = &connections[..min_len];

        let mut out1 = vec![Hoeg64Node { feature_mask: 0, adjacency_mask: 0, node_id: 0, node_type_hash: 0, _pad: [0; 44] }; min_len];
        let mut out2 = out1.clone();

        compile_hoeg_matrix(nt, ft, cn, &mut out1).unwrap();
        compile_hoeg_matrix(nt, ft, cn, &mut out2).unwrap();

        prop_assert!(hoeg_nodes_eq(&out1, &out2), "HOEG matrix compiler is not deterministic!");
    }

    #[test]
    fn prop_gnn_determinism(
        node_features in prop::collection::vec(any::<u64>(), 1..64),
        node_adjs in prop::collection::vec(any::<u64>(), 1..64),
        weight_vec in prop::collection::vec(any::<u64>(), 64..=64),
        bias in any::<u64>()
    ) {
        let min_len = node_features.len().min(node_adjs.len());
        let mut nodes = vec![Hoeg64Node { feature_mask: 0, adjacency_mask: 0, node_id: 0, node_type_hash: 0, _pad: [0; 44] }; min_len];
        for i in 0..min_len {
            nodes[i].feature_mask = node_features[i];
            nodes[i].adjacency_mask = node_adjs[i];
            nodes[i].node_id = i as u16;
        }
        let weight_arr: [u64; 64] = weight_vec.try_into().unwrap();
        let layer = BinarizedGnnLayer { weights: weight_arr, bias };

        let mut out1 = vec![0u64; min_len];
        let mut out2 = vec![0u64; min_len];

        hoeg_gnn_forward(&nodes, &layer, &mut out1).unwrap();
        hoeg_gnn_forward(&nodes, &layer, &mut out2).unwrap();

        prop_assert_eq!(&out1, &out2, "GNN forward is not deterministic!");
    }

    #[test]
    fn prop_tekg_determinism(
        entity_id in any::<u16>(),
        update_timestamps in prop::collection::vec(any::<u64>(), 1..64),
    ) {
        let mut out1 = vec![Tekg64Node { timestamp_ns: 0, rel_mask: 0, node_id: 0, parent_id: 0, prev_snapshot_id: 0, label: TekgLabel::Log, _pad: [0; 41] }; update_timestamps.len() + 1];
        let mut out2 = out1.clone();

        compile_snapshot_chain(entity_id, &update_timestamps, &mut out1).unwrap();
        compile_snapshot_chain(entity_id, &update_timestamps, &mut out2).unwrap();

        prop_assert!(tekg_nodes_eq(&out1, &out2), "TEKG chain is not deterministic!");
    }
}

// Falsification tests (prove the mutants are distinct from real and fail)
#[test]
fn test_falsify_hoeg_mutants() {
    let mut m1_caught = false;
    let mut m2_caught = false;
    let mut m3_caught = false;

    // We can just explicitly test an input that separates them
    let nt = [1, 2, 3];
    let ft = [0b101, 0b110, 0b011];
    let cn = [0b100, 0b010, 0b001];

    let mut out_real = [Hoeg64Node {
        feature_mask: 0,
        adjacency_mask: 0,
        node_id: 0,
        node_type_hash: 0,
        _pad: [0; 44],
    }; 3];
    compile_hoeg_matrix(&nt, &ft, &cn, &mut out_real).unwrap();

    let mut out_m1 = out_real.clone();
    mutant1_compile_hoeg_matrix(&nt, &ft, &cn, &mut out_m1).unwrap();
    if !hoeg_nodes_eq(&out_real, &out_m1) {
        m1_caught = true;
    }

    let mut out_m2 = out_real.clone();
    mutant2_compile_hoeg_matrix(&nt, &ft, &cn, &mut out_m2).unwrap();
    if !hoeg_nodes_eq(&out_real, &out_m2) {
        m2_caught = true;
    }

    let mut out_m3 = out_real.clone();
    mutant3_compile_hoeg_matrix(&nt, &ft, &cn, &mut out_m3).unwrap();
    if !hoeg_nodes_eq(&out_real, &out_m3) {
        m3_caught = true;
    }

    assert!(m1_caught, "HOEG Mutant 1 survived!");
    assert!(m2_caught, "HOEG Mutant 2 survived!");
    assert!(m3_caught, "HOEG Mutant 3 survived!");
}

#[test]
fn test_falsify_gnn_mutants() {
    let mut m1_caught = false;
    let mut m2_caught = false;
    let mut m3_caught = false;

    let nodes = [
        Hoeg64Node {
            feature_mask: 0b1,
            adjacency_mask: 0b1,
            node_id: 0,
            node_type_hash: 1,
            _pad: [0; 44],
        },
        // Node 1 will use weight 1 in real, weight 0 in mutant 2
        Hoeg64Node {
            feature_mask: 0,
            adjacency_mask: 0,
            node_id: 1,
            node_type_hash: 1,
            _pad: [0; 44],
        },
    ];
    let mut weights = [0u64; 64];
    weights[0] = 0; // xnor with 0 is !0, pop = 64 -> output varies based on lowest bit
    weights[1] = u64::MAX; // xnor with !0 is 0, pop = 0 -> mask = 0 -> output = bias
    let layer = BinarizedGnnLayer { weights, bias: 0b1100 };

    let mut out_real = [0u64; 2];
    hoeg_gnn_forward(&nodes, &layer, &mut out_real).unwrap();

    let mut out_m1 = [0u64; 2];
    mutant1_hoeg_gnn_forward(&nodes, &layer, &mut out_m1).unwrap();
    if out_real != out_m1 {
        m1_caught = true;
    }

    let mut out_m2 = [0u64; 2];
    mutant2_hoeg_gnn_forward(&nodes, &layer, &mut out_m2).unwrap();
    if out_real != out_m2 {
        m2_caught = true;
    }

    let mut out_m3 = [0u64; 2];
    mutant3_hoeg_gnn_forward(&nodes, &layer, &mut out_m3).unwrap();
    if out_real != out_m3 {
        m3_caught = true;
    }

    assert!(m1_caught, "GNN Mutant 1 survived! (No Adjacency Mixing)");
    assert!(m2_caught, "GNN Mutant 2 survived! (Always weight 0)");
    assert!(m3_caught, "GNN Mutant 3 survived! (Flipped bias)");
}

#[test]
fn test_falsify_tekg_mutants() {
    let mut m1_caught = false;
    let mut m2_caught = false;
    let mut m3_caught = false;

    let entity_id = 42;
    let timestamps = [100, 200, 300];
    let mut out_real = [Tekg64Node {
        timestamp_ns: 0,
        rel_mask: 0,
        node_id: 0,
        parent_id: 0,
        prev_snapshot_id: 0,
        label: TekgLabel::Log,
        _pad: [0; 41],
    }; 4];
    compile_snapshot_chain(entity_id, &timestamps, &mut out_real).unwrap();

    let mut out_m1 = out_real.clone();
    mutant1_compile_snapshot_chain(entity_id, &timestamps, &mut out_m1).unwrap();
    if !tekg_nodes_eq(&out_real, &out_m1) {
        m1_caught = true;
    }

    let mut out_m2 = out_real.clone();
    mutant2_compile_snapshot_chain(entity_id, &timestamps, &mut out_m2).unwrap();
    if !tekg_nodes_eq(&out_real, &out_m2) {
        m2_caught = true;
    }

    let mut out_m3 = out_real.clone();
    mutant3_compile_snapshot_chain(entity_id, &timestamps, &mut out_m3).unwrap();
    if !tekg_nodes_eq(&out_real, &out_m3) {
        m3_caught = true;
    }

    assert!(m1_caught, "TEKG Mutant 1 survived!");
    assert!(m2_caught, "TEKG Mutant 2 survived!");
    assert!(m3_caught, "TEKG Mutant 3 survived!");
}
