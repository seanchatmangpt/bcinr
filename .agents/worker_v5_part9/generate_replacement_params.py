import os
import json
import re

PARTITION_9 = {
    "delta_encode_simd_u32": (
        "val.wrapping_sub(aux)",
        "(val as i128 - aux as i128) as u64"
    ),
    "delta_decode_simd_u32": (
        "val.wrapping_add(aux)",
        "(val as i128 + aux as i128) as u64"
    ),
    "branchless_stack_spsc": (
        "(val.wrapping_add(1)) & aux",
        "let next = val.wrapping_add(1); next & aux"
    ),
    "branchless_ring_buffer_mpmc": (
        "(val.wrapping_add(1)) & aux",
        "let next = val.wrapping_add(1); next & aux"
    ),
    "lockfree_skip_list_step": (
        "(aux > val) as u64",
        "if aux > val { 1 } else { 0 }"
    ),
    "waitfree_queue_push": (
        "let tail = val & 0xFFFFFFFF; let new_tail = tail.wrapping_add(1); (val & 0xFFFFFFFF00000000) | new_tail",
        "let tail = val & 0xFFFFFFFF; (val & 0xFFFFFFFF00000000) | (tail.wrapping_add(1))"
    ),
    "hazard_pointer_retire": (
        "val ^ aux.wrapping_add(0xDEADBEEF)",
        "let offset = aux.wrapping_add(0xDEADBEEF); val ^ offset"
    ),
    "epoch_based_reclamation_step": (
        "val.wrapping_add(1) & (aux.wrapping_neg() | aux)",
        "if aux != 0 { val.wrapping_add(1) } else { 0 }"
    ),
    "branchless_priority_queue_push": (
        "let mask = 0u64.wrapping_sub((val < aux) as u64); (val & !mask) | (aux & mask)",
        "if val > aux { val } else { aux }"
    ),
    "branchless_priority_queue_pop": (
        "let mask = 0u64.wrapping_sub((val > aux) as u64); (val & !mask) | (aux & mask)",
        "if val < aux { val } else { aux }"
    ),
    "disjoint_set_union_branchless": (
        "let is_root = (val == aux) as u64; (is_root.wrapping_neg() & val) | ((!is_root.wrapping_neg()) & aux)",
        "if val == aux { val } else { aux }"
    ),
    "graph_bfs_simd_step": (
        "val & !aux",
        "let unvisited = !aux; val & unvisited"
    ),
    "graph_dfs_bit_parallel": (
        "let unvisited = val & !aux; unvisited & unvisited.wrapping_neg()",
        "let unvisited = val & !aux; if unvisited == 0 { 0 } else { 1u64.wrapping_shl(unvisited.trailing_zeros() as u32) }"
    ),
    "shortest_path_bellman_ford_branchless": (
        "val.saturating_add(aux)",
        "if val > u64::MAX - aux { u64::MAX } else { val + aux }"
    ),
    "page_rank_simd_step": (
        "let nz = (aux != 0) as u64; let out_degree = (aux + (1 - nz)) as f64; let rank = (val * nz) as f64; (rank / out_degree) as u64",
        "if aux == 0 { 0 } else { (val as f64 / aux as f64) as u64 }"
    ),
    "triangle_count_bitset": (
        "(val & aux).count_ones() as u64",
        "let mut count = 0; for i in 0..64 { if ((val & aux) & (1 << i)) != 0 { count += 1; } } count"
    ),
    "clique_check_branchless": (
        "((val & aux) == val) as u64",
        "if (val & aux) == val { 1 } else { 0 }"
    ),
    "topological_sort_step_branchless": (
        "val & !aux",
        "let mask = !aux; val & mask"
    ),
    "minimum_spanning_tree_prim_step": (
        "let m = 0u64.wrapping_sub((val < aux) as u64); (val & m) | (aux & !m)",
        "if val < aux { val } else { aux }"
    ),
    "max_flow_edmonds_karp_step": (
        "let cap = val; let flow = aux; let valid = (cap >= flow) as u64; (cap.wrapping_sub(flow)) * valid",
        "if val >= aux { val - aux } else { 0 }"
    ),
    "bloom_filter_graph_visited": (
        "val | (1u64 << (aux & 63))",
        "let bit = 1u64 << (aux & 63); val | bit"
    ),
    "matrix_mul_simd_f32": (
        "let a1 = f32::from_bits((val & 0xFFFFFFFF) as u32); let a2 = f32::from_bits((val >> 32) as u32); let b1 = f32::from_bits((aux & 0xFFFFFFFF) as u32); let b2 = f32::from_bits((aux >> 32) as u32); (a1 * b1 + a2 * b2).to_bits() as u64",
        "let a1 = f32::from_bits((val & 0xFFFFFFFF) as u32); let a2 = f32::from_bits((val >> 32) as u32); let b1 = f32::from_bits((aux & 0xFFFFFFFF) as u32); let b2 = f32::from_bits((aux >> 32) as u32); let sum = (a1 * b1) + (a2 * b2); sum.to_bits() as u64"
    ),
    "matrix_transpose_simd_f32": (
        "let a11 = val & 0xFFFFFFFF; let a21 = aux & 0xFFFFFFFF; a11 | (a21 << 32)",
        "let a11 = val & 0xFFFFFFFF; let a21 = aux & 0xFFFFFFFF; (a21 << 32) | a11"
    ),
    "vector_dot_product_simd_f32": (
        "let ax = f32::from_bits((val & 0xFFFFFFFF) as u32); let ay = f32::from_bits((val >> 32) as u32); let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32); let by = f32::from_bits((aux >> 32) as u32); let dot = ax * bx + ay * by; dot.to_bits() as u64",
        "let ax = f32::from_bits((val & 0xFFFFFFFF) as u32); let ay = f32::from_bits((val >> 32) as u32); let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32); let by = f32::from_bits((aux >> 32) as u32); let dot = (ax * bx) + (ay * by); dot.to_bits() as u64"
    ),
    "vector_cross_product_f32": (
        "let ax = f32::from_bits((val & 0xFFFFFFFF) as u32); let ay = f32::from_bits((val >> 32) as u32); let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32); let by = f32::from_bits((aux >> 32) as u32); let cross = ax * by - ay * bx; cross.to_bits() as u64",
        "let ax = f32::from_bits((val & 0xFFFFFFFF) as u32); let ay = f32::from_bits((val >> 32) as u32); let bx = f32::from_bits((aux & 0xFFFFFFFF) as u32); let by = f32::from_bits((aux >> 32) as u32); let cross = (ax * by) - (ay * bx); cross.to_bits() as u64"
    ),
    "quaternion_mul_branchless": (
        "let a = val >> 32; let b = val & 0xFFFFFFFF; let c = aux >> 32; let d = aux & 0xFFFFFFFF; let r = a.wrapping_mul(c).wrapping_sub(b.wrapping_mul(d)); let i = a.wrapping_mul(d).wrapping_add(b.wrapping_mul(c)); (r << 32) | (i & 0xFFFFFFFF)",
        "let a = val >> 32; let b = val & 0xFFFFFFFF; let c = aux >> 32; let d = aux & 0xFFFFFFFF; let r = (a.wrapping_mul(c)).wrapping_sub(b.wrapping_mul(d)); let i = (a.wrapping_mul(d)).wrapping_add(b.wrapping_mul(c)); (r << 32) | (i & 0xFFFFFFFF)"
    ),
    "aabb_intersect_branchless": (
        "let x1_min = val & 0xFFFF; let x1_max = (val >> 16) & 0xFFFF; let y1_min = (val >> 32) & 0xFFFF; let y1_max = (val >> 48) & 0xFFFF; let x2_min = aux & 0xFFFF; let x2_max = (aux >> 16) & 0xFFFF; let y2_min = (aux >> 32) & 0xFFFF; let y2_max = (aux >> 48) & 0xFFFF; ((x1_min <= x2_max) & (x2_min <= x1_max) & (y1_min <= y2_max) & (y2_min <= y1_max)) as u64",
        "let x1_min = val & 0xFFFF; let x1_max = (val >> 16) & 0xFFFF; let y1_min = (val >> 32) & 0xFFFF; let y1_max = (val >> 48) & 0xFFFF; let x2_min = aux & 0xFFFF; let x2_max = (aux >> 16) & 0xFFFF; let y2_min = (aux >> 32) & 0xFFFF; let y2_max = (aux >> 48) & 0xFFFF; if x1_min <= x2_max && x2_min <= x1_max && y1_min <= y2_max && y2_min <= y1_max { 1 } else { 0 }"
    ),
    "ray_triangle_intersect_branchless": (
        "let det = val.wrapping_mul(aux); let inv_det = 1u64.wrapping_div(det | (det == 0) as u64); inv_det * (det != 0) as u64",
        "let det = val.wrapping_mul(aux); if det == 0 { 0 } else { 1u64.wrapping_div(det) }"
    ),
    "ray_sphere_intersect_branchless": (
        "let b = val; let c = aux; let disc = b.wrapping_mul(b).wrapping_sub(4u64.wrapping_mul(c)); (disc.leading_zeros() == 0) as u64",
        "let b = val; let c = aux; let disc = b.wrapping_mul(b).wrapping_sub(4u64.wrapping_mul(c)); if (disc >> 63) == 1 { 0 } else { 1 }"
    ),
    "frustum_culling_branchless": (
        "let x = (val >> 32) as i32; let y = val as i32; let min_x = (aux >> 48) as i16 as i32; let max_x = ((aux >> 32) & 0xFFFF) as i16 as i32; let min_y = ((aux >> 16) & 0xFFFF) as i16 as i32; let max_y = (aux & 0xFFFF) as i16 as i32; ((x >= min_x) & (x <= max_x) & (y >= min_y) & (y <= max_y)) as u64",
        "let x = (val >> 32) as i32; let y = val as i32; let min_x = (aux >> 48) as i16 as i32; let max_x = ((aux >> 32) & 0xFFFF) as i16 as i32; let min_y = ((aux >> 16) & 0xFFFF) as i16 as i32; let max_y = (aux & 0xFFFF) as i16 as i32; if x >= min_x && x <= max_x && y >= min_y && y <= max_y { 1 } else { 0 }"
    ),
    "point_in_polygon_branchless": (
        "let py = (val >> 32) as i32; let px = (val & 0xFFFFFFFF) as i32; let v1x = (aux & 0xFFFF) as i32; let v1y = ((aux >> 16) & 0xFFFF) as i32; let v2x = ((aux >> 32) & 0xFFFF) as i32; let v2y = (aux >> 48) as i32; let cond1 = (v1y > py) != (v2y > py); let denom = v2y - v1y + (v2y == v1y) as i32; let intersect = cond1 & (px < (v2x - v1x) * (py - v1y) / denom + v1x); intersect as u64",
        "let py = (val >> 32) as i32; let px = (val & 0xFFFFFFFF) as i32; let v1x = (aux & 0xFFFF) as i32; let v1y = ((aux >> 16) & 0xFFFF) as i32; let v2x = ((aux >> 32) & 0xFFFF) as i32; let v2y = (aux >> 48) as i32; if (v1y > py) != (v2y > py) { if px < (v2x - v1x) * (py - v1y) / (v2y - v1y) + v1x { 1 } else { 0 } } else { 0 }"
    )
}

src_dir = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"
dest_payloads = {}

for name, (impl_code, ref_code) in PARTITION_9.items():
    path = os.path.join(src_dir, f"{name}.rs")
    with open(path, "r") as f:
        content = f.read()
    
    # Let's locate the start line and end line of the target block
    # Start: the line with "/// # CONTRACT" (or "/// # Branchless Contract")
    # End: the closing brace of the reference function
    lines = content.splitlines()
    start_line = None
    end_line = None
    
    for idx, line in enumerate(lines):
        if "/// # CONTRACT" in line or "/// # Branchless Contract" in line:
            start_line = idx + 1
            break
            
    # Find the reference function signature
    ref_fn_sig = f"fn {name}_reference"
    ref_start_idx = None
    for idx, line in enumerate(lines):
        if ref_fn_sig in line:
            ref_start_idx = idx
            break
            
    # Find the closing brace of reference function
    # It should be the first line with just "    }" after ref_start_idx
    for idx in range(ref_start_idx, len(lines)):
        if lines[idx].strip() == "}":
            end_line = idx + 1
            break
            
    if start_line is None or end_line is None:
        print(f"Error locating block in {name}.rs: start={start_line}, end={end_line}")
        continue
        
    # Get exact target content
    target_content = "\n".join(lines[start_line - 1 : end_line])
    
    # Construct replacement content
    # Replace the body of name(...) and name_reference(...)
    # Let's read the updated file from new_rs_files to get the exact clean block
    updated_path = os.path.join("/Users/sac/bcinr/.agents/worker_v5_part9/new_rs_files", f"{name}.rs")
    with open(updated_path, "r") as f:
        up_content = f.read()
    up_lines = up_content.splitlines()
    
    # Find start and end in updated lines
    up_start_line = None
    up_end_line = None
    for idx, line in enumerate(up_lines):
        if "/// # Branchless Contract" in line:
            up_start_line = idx + 1
            break
    for idx in range(up_start_line, len(up_lines)):
        if up_lines[idx].strip() == "}":
            up_end_line = idx + 1
            if f"fn {name}_reference" in "\n".join(up_lines[up_start_line-1 : idx+1]):
                break
                
    replacement_content = "\n".join(up_lines[up_start_line - 1 : up_end_line])
    
    dest_payloads[name] = {
        "TargetFile": path,
        "StartLine": start_line,
        "EndLine": end_line,
        "TargetContent": target_content,
        "ReplacementContent": replacement_content
    }

with open("/Users/sac/bcinr/.agents/worker_v5_part9/replace_payloads.json", "w") as f:
    json.dump(dest_payloads, f, indent=2)
print("Done writing replacement payloads.")
