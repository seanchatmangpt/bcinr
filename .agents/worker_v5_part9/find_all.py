import os

PARTITION_9 = [
    "delta_encode_simd_u32",
    "delta_decode_simd_u32",
    "branchless_stack_spsc",
    "branchless_ring_buffer_mpmc",
    "lockfree_skip_list_step",
    "waitfree_queue_push",
    "hazard_pointer_retire",
    "epoch_based_reclamation_step",
    "branchless_priority_queue_push",
    "branchless_priority_queue_pop",
    "disjoint_set_union_branchless",
    "graph_bfs_simd_step",
    "graph_dfs_bit_parallel",
    "shortest_path_bellman_ford_branchless",
    "page_rank_simd_step",
    "triangle_count_bitset",
    "clique_check_branchless",
    "topological_sort_step_branchless",
    "minimum_spanning_tree_prim_step",
    "max_flow_edmonds_karp_step",
    "bloom_filter_graph_visited",
    "matrix_mul_simd_f32",
    "matrix_transpose_simd_f32",
    "vector_dot_product_simd_f32",
    "vector_cross_product_f32",
    "quaternion_mul_branchless",
    "aabb_intersect_branchless",
    "ray_triangle_intersect_branchless",
    "ray_sphere_intersect_branchless",
    "frustum_culling_branchless",
    "point_in_polygon_branchless"
]

root_dir = "/Users/sac/bcinr"

for algo in PARTITION_9:
    print(f"=== {algo} ===")
    matches = []
    for root, dirs, files in os.walk(root_dir):
        if ".git" in root or ".agents" in root or "target" in root:
            continue
        for file in files:
            if not file.endswith(".py") and not file.endswith(".rs") and not file.endswith(".txt"):
                continue
            path = os.path.join(root, file)
            try:
                with open(path, "r", errors="ignore") as f:
                    content = f.read()
                if algo in content:
                    lines = content.splitlines()
                    for idx, line in enumerate(lines):
                        if algo in line and ("def " in line or "fn " in line or " = " in line or ":" in line or "(" in line):
                            # Skip if it is just a bench or test in bench files
                            if "benches" in path or "all_300_bench" in path:
                                continue
                            matches.append((path, idx + 1, line.strip()))
            except Exception as e:
                pass
    
    # print unique matches up to 10
    seen = set()
    count = 0
    for path, line_no, line in matches:
        key = (path, line)
        if key not in seen:
            seen.add(key)
            print(f"  {path}:{line_no} -> {line}")
            count += 1
            if count >= 15:
                break
    if count == 0:
        print("  NOT FOUND")
    print()
