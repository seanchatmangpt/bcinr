import os

MISSING = [
    "waitfree_queue_push",
    "hazard_pointer_retire",
    "branchless_priority_queue_push",
    "branchless_priority_queue_pop",
    "graph_bfs_simd_step",
    "graph_dfs_bit_parallel",
    "page_rank_simd_step",
    "triangle_count_bitset",
    "topological_sort_step_branchless",
    "vector_dot_product_simd_f32",
    "vector_cross_product_f32",
    "aabb_intersect_branchless",
    "frustum_culling_branchless"
]

root_dir = "/Users/sac/bcinr"
out_file = "/Users/sac/bcinr/.agents/worker_v5_part9/missing_found.txt"

with open(out_file, "w") as out:
    for algo in MISSING:
        out.write(f"=== {algo} ===\n")
        matches = []
        for root, dirs, files in os.walk(root_dir):
            if ".git" in root or ".agents" in root or "target" in root:
                continue
            for file in files:
                if not file.endswith(".py"):
                    continue
                path = os.path.join(root, file)
                try:
                    with open(path, "r", errors="ignore") as f:
                        content = f.read()
                    if algo in content:
                        lines = content.splitlines()
                        for idx, line in enumerate(lines):
                            if algo in line:
                                matches.append((path, idx + 1))
                except Exception as e:
                    pass
        
        for path, line_no in matches:
            out.write(f"  Found in {path}:{line_no}\n")
            # print 12 lines of context
            with open(path, "r", errors="ignore") as f:
                lines = f.readlines()
            start = max(0, line_no - 3)
            end = min(len(lines), line_no + 12)
            for i in range(start, end):
                out.write(f"    {i+1}: {lines[i].rstrip()}\n")
        out.write("\n")
