import os
import ast

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

# We will read each implement_*.py file and search for the algorithm names
# inside Python AST to find dictionary values or variable assignments.
for fn in sorted(os.listdir(root_dir)):
    if not fn.startswith("implement") or not fn.endswith(".py"):
        continue
    path = os.path.join(root_dir, fn)
    with open(path, "r", errors="ignore") as f:
        code = f.read()
    
    try:
        tree = ast.parse(code)
    except Exception as e:
        print(f"AST error in {fn}: {e}")
        continue
        
    class Finder(ast.NodeVisitor):
        def visit_Dict(self, node):
            # Check if any key is in Partition 9
            for k, v in zip(node.keys, node.values):
                if isinstance(k, ast.Constant) and k.value in PARTITION_9:
                    print(f"File: {fn}")
                    print(f"Algo: {k.value}")
                    if isinstance(v, ast.Tuple):
                        for el in v.elts:
                            if isinstance(el, ast.Constant):
                                print(f"  Tuple element: {repr(el.value)}")
                    elif isinstance(v, ast.Constant):
                        print(f"  Val: {repr(v.value)}")
                    elif isinstance(v, ast.Dict):
                        for sub_k, sub_v in zip(v.keys, v.values):
                            if isinstance(sub_k, ast.Constant) and isinstance(sub_v, ast.Constant):
                                print(f"  {sub_k.value}: {repr(sub_v.value)}")
                    print("-" * 50)
            self.generic_visit(node)
            
        def visit_Assign(self, node):
            # Check for pattern: algos = { ... } or similar
            self.generic_visit(node)
            
    Finder().visit(tree)
