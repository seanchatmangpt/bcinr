import os
import ast

part2_algos = [
    "is_contiguous_mask_u64", "get_mask_boundary_low_u64", "get_mask_boundary_high_u64",
    "bit_matrix_transpose_8x8", "bit_matrix_transpose_64x64", "rank_u128", "select_u128",
    "weight_u64", "delta_swap_u64", "benes_network_u64", "bit_permute_step_u64",
    "compress_bits_u64", "expand_bits_u64", "crossbar_permute_u8x16", "mask_from_bool_slice",
    "bool_slice_from_mask", "bit_permute_identity_64", "is_subset_mask_u64",
    "mask_xor_reduce_u64", "mul_sat_u64", "div_sat_u64", "add_sat_i32", "sub_sat_i32",
    "mul_sat_i32", "abs_diff_u64", "abs_diff_i64", "avg_u64", "avg_ceil_u64",
    "clamp_i64", "lerp_sat_u8", "lerp_sat_u32"
]

found_algos = {}

def get_val(node):
    if isinstance(node, ast.Constant):
        return node.value
    elif isinstance(node, ast.Str):
        return node.s
    return None

files = [f for f in os.listdir("/Users/sac/bcinr") if f.endswith(".py")]

for fname in sorted(files):
    path = os.path.join("/Users/sac/bcinr", fname)
    with open(path, "r", encoding="utf-8", errors="ignore") as f:
        code = f.read()
    
    try:
        tree = ast.parse(code)
        for node in ast.walk(tree):
            if isinstance(node, ast.Assign):
                # check for lists/dicts
                if isinstance(node.value, ast.List):
                    for elt in node.value.elts:
                        if isinstance(elt, ast.Tuple):
                            vals = [get_val(v) for v in elt.elts if get_val(v) is not None]
                            if vals:
                                name = vals[0]
                                if name in part2_algos:
                                    if len(vals) == 4:
                                        found_algos[name] = (vals[2], vals[3], fname)
                                    elif len(vals) == 3:
                                        found_algos[name] = (vals[1], vals[2], fname)
                elif isinstance(node.value, ast.Dict):
                    for k, v in zip(node.value.keys, node.value.values):
                        k_val = get_val(k)
                        if k_val in part2_algos:
                            if isinstance(v, ast.Tuple):
                                vals = [get_val(val_node) for val_node in v.elts if get_val(val_node) is not None]
                                if len(vals) == 2:
                                    found_algos[k_val] = (vals[0], vals[1], fname)
                                elif len(vals) == 3:
                                    found_algos[k_val] = (vals[1], vals[2], fname)
                            else:
                                v_val = get_val(v)
                                if v_val is not None:
                                    found_algos[k_val] = (v_val, v_val, fname)
    except Exception as e:
        pass

with open("/Users/sac/bcinr/.agents/worker_v5_part2/exact_lookup.txt", "w") as out:
    for name in part2_algos:
        if name in found_algos:
            impl, ref, origin = found_algos[name]
            out.write(f"ALGO_NAME: {name}\n")
            out.write(f"ORIGIN: {origin}\n")
            out.write(f"IMPL_BODY:\n{impl}\n")
            out.write(f"REF_BODY:\n{ref}\n")
            out.write("-" * 80 + "\n")
        else:
            out.write(f"ALGO_NAME: {name}\n")
            out.write("NOT FOUND\n")
            out.write("-" * 80 + "\n")
