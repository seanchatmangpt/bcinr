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
python_files = [f for f in os.listdir(root_dir) if f.startswith("implement") and f.endswith(".py")]
python_files.extend(["refine_all_batches.py", "refine_batch_2_3.py"])

for algo in PARTITION_9:
    print(f"=== {algo} ===")
    found = False
    for py_file in python_files:
        path = os.path.join(root_dir, py_file)
        if not os.path.exists(path):
            continue
        with open(path, "r", errors="ignore") as f:
            lines = f.readlines()
        for idx, line in enumerate(lines):
            if algo in line:
                print(f"  {py_file}:{idx+1}: {line.strip()}")
                # Print a few lines context if it is implement_*.py
                if "implement" in py_file:
                    start = max(0, idx - 2)
                    end = min(len(lines), idx + 8)
                    for j in range(start, end):
                        print(f"    {j+1}: {lines[j].rstrip()}")
                found = True
    if not found:
        print("  NOT FOUND ANYWHERE")
    print()
