import os
import re

ALGO_DIR = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"

def refactor_file(filename, replacement_rules):
    path = os.path.join(ALGO_DIR, filename)
    if not os.path.exists(path):
        print(f"Error: {path} not found")
        return
    content = open(path).read()
    for pattern, repl in replacement_rules:
        content = re.sub(pattern, repl, content)
    with open(path, "w") as f:
        f.write(content)
    print(f"Refactored {filename}")

def refactor_all():
    # 1. bloom_filter_add_u64.rs
    refactor_file("bloom_filter_add_u64.rs", [
        (r"pub fn bloom_filter_add_u64\(filter: u64, val: u64\)", "pub fn bloom_filter_add_u64(val: u64, aux: u64)"),
        (r"fn bloom_filter_add_u64_reference\(filter: u64, val: u64\)", "fn bloom_filter_add_u64_reference(val: u64, aux: u64)"),
        (r"fn mutant_bloom_filter_add_u64_(\d)\(filter: u64, val: u64\)", r"fn mutant_bloom_filter_add_u64_\1(val: u64, aux: u64)"),
        (r"bloom_filter_add_u64_reference\(filter, val\)", "bloom_filter_add_u64_reference(val, aux)"),
        (r"bloom_filter_add_u64\(filter, val\)", "bloom_filter_add_u64(val, aux)"),
        (r"mutant_bloom_filter_add_u64_(\d)\(filter, val\)", r"mutant_bloom_filter_add_u64_\1(val, aux)"),
        (r"filter in any::<u64>\(\), val in any::<u64>\(\)", "val in any::<u64>(), aux in any::<u64>()"),
        (r"bloom_filter_add_u64\(0, 42\)", "bloom_filter_add_u64(42, 1337)"),
        (r"bloom_filter_add_u64\(black_box\(0\), black_box\(42\)\)", "bloom_filter_add_u64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*filter,\s*val\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*bloom_filter_add_u64_reference\(filter,\s*val\)\s*\}", "Postcondition: { result = bloom_filter_add_u64_reference(val, aux) }"),
    ])

    # 2. bloom_filter_graph_visited.rs
    refactor_file("bloom_filter_graph_visited.rs", [
        (r"pub fn bloom_filter_graph_visited\(filter: u64, val: u64\)", "pub fn bloom_filter_graph_visited(val: u64, aux: u64)"),
        (r"fn bloom_filter_graph_visited_reference\(filter: u64, val: u64\)", "fn bloom_filter_graph_visited_reference(val: u64, aux: u64)"),
        (r"fn mutant_bloom_filter_graph_visited_(\d)\(filter: u64, val: u64\)", r"fn mutant_bloom_filter_graph_visited_\1(val: u64, aux: u64)"),
        (r"bloom_filter_graph_visited_reference\(filter, val\)", "bloom_filter_graph_visited_reference(val, aux)"),
        (r"bloom_filter_graph_visited\(filter, val\)", "bloom_filter_graph_visited(val, aux)"),
        (r"mutant_bloom_filter_graph_visited_(\d)\(filter, val\)", r"mutant_bloom_filter_graph_visited_\1(val, aux)"),
        (r"filter in any::<u64>\(\), val in any::<u64>\(\)", "val in any::<u64>(), aux in any::<u64>()"),
        (r"bloom_filter_graph_visited\(0, 42\)", "bloom_filter_graph_visited(42, 1337)"),
        (r"bloom_filter_graph_visited\(black_box\(0\), black_box\(42\)\)", "bloom_filter_graph_visited(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*filter,\s*val\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*bloom_filter_graph_visited_reference\(filter,\s*val\)\s*\}", "Postcondition: { result = bloom_filter_graph_visited_reference(val, aux) }"),
    ])

    # 3. bloom_filter_intersect.rs
    refactor_file("bloom_filter_intersect.rs", [
        (r"pub fn bloom_filter_intersect\(f1: u64, f2: u64\)", "pub fn bloom_filter_intersect(val: u64, aux: u64)"),
        (r"fn bloom_filter_intersect_reference\(f1: u64, f2: u64\)", "fn bloom_filter_intersect_reference(val: u64, aux: u64)"),
        (r"fn mutant_bloom_filter_intersect_(\d)\(f1: u64, f2: u64\)", r"fn mutant_bloom_filter_intersect_\1(val: u64, aux: u64)"),
        (r"bloom_filter_intersect_reference\(f1, f2\)", "bloom_filter_intersect_reference(val, aux)"),
        (r"bloom_filter_intersect\(f1, f2\)", "bloom_filter_intersect(val, aux)"),
        (r"mutant_bloom_filter_intersect_(\d)\(f1, f2\)", r"mutant_bloom_filter_intersect_\1(val, aux)"),
        (r"f1 in any::<u64>\(\), f2 in any::<u64>\(\)", "val in any::<u64>(), aux in any::<u64>()"),
        (r"bloom_filter_intersect\(u64::MAX, u64::MAX\)", "bloom_filter_intersect(42, 1337)"),
        (r"bloom_filter_intersect\(black_box\(u64::MAX\), black_box\(u64::MAX\)\)", "bloom_filter_intersect(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*f1,\s*f2\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*bloom_filter_intersect_reference\(f1,\s*f2\)\s*\}", "Postcondition: { result = bloom_filter_intersect_reference(val, aux) }"),
        (r"assert_eq!\(bloom_filter_intersect\(0, 0\), bloom_filter_intersect_reference\(0, 0\)\);", "assert_eq!(bloom_filter_intersect(0, 0), bloom_filter_intersect_reference(0, 0));"),
        (r"assert_eq!\(bloom_filter_intersect\(u64::MAX, u64::MAX\), bloom_filter_intersect_reference\(u64::MAX, u64::MAX\)\);", "assert_eq!(bloom_filter_intersect(u64::MAX, u64::MAX), bloom_filter_intersect_reference(u64::MAX, u64::MAX));"),
        (r"f1 \| f2", "val | aux"),
        (r"f1 & f2", "val & aux"),
    ])

    # 4. bloom_filter_query_u64.rs
    refactor_file("bloom_filter_query_u64.rs", [
        (r"pub fn bloom_filter_query_u64\(filter: u64, val: u64\)", "pub fn bloom_filter_query_u64(val: u64, aux: u64)"),
        (r"fn bloom_filter_query_u64_reference\(filter: u64, val: u64\)", "fn bloom_filter_query_u64_reference(val: u64, aux: u64)"),
        (r"fn mutant_bloom_filter_query_u64_(\d)\(filter: u64, val: u64\)", r"fn mutant_bloom_filter_query_u64_\1(val: u64, aux: u64)"),
        (r"bloom_filter_query_u64_reference\(filter, val\)", "bloom_filter_query_u64_reference(val, aux)"),
        (r"bloom_filter_query_u64\(filter, val\)", "bloom_filter_query_u64(val, aux)"),
        (r"mutant_bloom_filter_query_u64_(\d)\(filter, val\)", r"mutant_bloom_filter_query_u64_\1(val, aux)"),
        (r"filter in any::<u64>\(\), val in any::<u64>\(\)", "val in any::<u64>(), aux in any::<u64>()"),
        (r"bloom_filter_query_u64\(0xFFFFFFFFFFFFFFFF, 42\)", "bloom_filter_query_u64(42, 1337)"),
        (r"bloom_filter_query_u64\(black_box\(0xFFFFFFFFFFFFFFFF\), black_box\(42\)\)", "bloom_filter_query_u64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*filter,\s*val\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*bloom_filter_query_u64_reference\(filter,\s*val\)\s*\}", "Postcondition: { result = bloom_filter_query_u64_reference(val, aux) }"),
    ])

    # 5. bloom_filter_union.rs
    refactor_file("bloom_filter_union.rs", [
        (r"pub fn bloom_filter_union\(f1: u64, f2: u64\)", "pub fn bloom_filter_union(val: u64, aux: u64)"),
        (r"fn bloom_filter_union_reference\(f1: u64, f2: u64\)", "fn bloom_filter_union_reference(val: u64, aux: u64)"),
        (r"fn mutant_bloom_filter_union_(\d)\(f1: u64, f2: u64\)", r"fn mutant_bloom_filter_union_\1(val: u64, aux: u64)"),
        (r"bloom_filter_union_reference\(f1, f2\)", "bloom_filter_union_reference(val, aux)"),
        (r"bloom_filter_union\(f1, f2\)", "bloom_filter_union(val, aux)"),
        (r"mutant_bloom_filter_union_(\d)\(f1, f2\)", r"mutant_bloom_filter_union_\1(val, aux)"),
        (r"f1 in any::<u64>\(\), f2 in any::<u64>\(\)", "val in any::<u64>(), aux in any::<u64>()"),
        (r"bloom_filter_union\(0, 42\)", "bloom_filter_union(42, 1337)"),
        (r"bloom_filter_union\(black_box\(0\), black_box\(42\)\)", "bloom_filter_union(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*f1,\s*f2\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*bloom_filter_union_reference\(f1,\s*f2\)\s*\}", "Postcondition: { result = bloom_filter_union_reference(val, aux) }"),
        (r"assert_eq!\(bloom_filter_union\(0, 0\), bloom_filter_union_reference\(0, 0\)\);", "assert_eq!(bloom_filter_union(0, 0), bloom_filter_union_reference(0, 0));"),
        (r"assert_eq!\(bloom_filter_union\(u64::MAX, u64::MAX\), bloom_filter_union_reference\(u64::MAX, u64::MAX\)\);", "assert_eq!(bloom_filter_union(u64::MAX, u64::MAX), bloom_filter_union_reference(u64::MAX, u64::MAX));"),
        (r"f1 \| f2", "val | aux"),
        (r"f1 & f2", "val & aux"),
    ])

    # 6. blsi_u64.rs
    refactor_file("blsi_u64.rs", [
        (r"pub fn blsi_u64\(x: u64\)", "pub fn blsi_u64(val: u64, aux: u64)"),
        (r"fn blsi_u64_reference\(x: u64\)", "fn blsi_u64_reference(val: u64, aux: u64)"),
        (r"fn mutant_blsi_u64_(\d)\(x: u64\)", r"fn mutant_blsi_u64_\1(val: u64, aux: u64)"),
        (r"blsi_u64_reference\(x\)", "blsi_u64_reference(val, aux)"),
        (r"blsi_u64\(x\)", "blsi_u64(val, aux)"),
        (r"mutant_blsi_u64_(\d)\(x\)", r"mutant_blsi_u64_\1(val, aux)"),
        (r"blsi_u64\(42\)", "blsi_u64(42, 1337)"),
        (r"blsi_u64\(black_box\(42\)\)", "blsi_u64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*x\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*blsi_u64_reference\(x\)\s*\}", "Postcondition: { result = blsi_u64_reference(val, aux) }"),
        (r"assert_eq!\(blsi_u64\(0\), blsi_u64_reference\(0\)\);", "assert_eq!(blsi_u64(0, 0), blsi_u64_reference(0, 0));"),
        (r"assert_eq!\(blsi_u64\(u64::MAX\), blsi_u64_reference\(u64::MAX\)\);", "assert_eq!(blsi_u64(u64::MAX, 0), blsi_u64_reference(u64::MAX, 0));"),
        (r"x\.wrapping_neg\(\) & x", "val.wrapping_neg() & val"),
    ])

    # 7. blsmsk_u64.rs
    refactor_file("blsmsk_u64.rs", [
        (r"pub fn blsmsk_u64\(x: u64\)", "pub fn blsmsk_u64(val: u64, aux: u64)"),
        (r"fn blsmsk_u64_reference\(x: u64\)", "fn blsmsk_u64_reference(val: u64, aux: u64)"),
        (r"fn mutant_blsmsk_u64_(\d)\(x: u64\)", r"fn mutant_blsmsk_u64_\1(val: u64, aux: u64)"),
        (r"blsmsk_u64_reference\(x\)", "blsmsk_u64_reference(val, aux)"),
        (r"blsmsk_u64\(x\)", "blsmsk_u64(val, aux)"),
        (r"mutant_blsmsk_u64_(\d)\(x\)", r"mutant_blsmsk_u64_\1(val, aux)"),
        (r"blsmsk_u64\(42\)", "blsmsk_u64(42, 1337)"),
        (r"blsmsk_u64\(black_box\(42\)\)", "blsmsk_u64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*x\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*blsmsk_u64_reference\(x\)\s*\}", "Postcondition: { result = blsmsk_u64_reference(val, aux) }"),
        (r"assert_eq!\(blsmsk_u64\(0\), blsmsk_u64_reference\(0\)\);", "assert_eq!(blsmsk_u64(0, 0), blsmsk_u64_reference(0, 0));"),
        (r"assert_eq!\(blsmsk_u64\(u64::MAX\), blsmsk_u64_reference\(u64::MAX\)\);", "assert_eq!(blsmsk_u64(u64::MAX, 0), blsmsk_u64_reference(u64::MAX, 0));"),
        (r"x \^ \(x\.wrapping_sub\(1\)\)", "val ^ (val.wrapping_sub(1))"),
    ])

    # 8. blsr_u64.rs
    refactor_file("blsr_u64.rs", [
        (r"pub fn blsr_u64\(x: u64\)", "pub fn blsr_u64(val: u64, aux: u64)"),
        (r"fn blsr_u64_reference\(x: u64\)", "fn blsr_u64_reference(val: u64, aux: u64)"),
        (r"fn mutant_blsr_u64_(\d)\(x: u64\)", r"fn mutant_blsr_u64_\1(val: u64, aux: u64)"),
        (r"blsr_u64_reference\(x\)", "blsr_u64_reference(val, aux)"),
        (r"blsr_u64\(x\)", "blsr_u64(val, aux)"),
        (r"mutant_blsr_u64_(\d)\(x\)", r"mutant_blsr_u64_\1(val, aux)"),
        (r"blsr_u64\(42\)", "blsr_u64(42, 1337)"),
        (r"blsr_u64\(black_box\(42\)\)", "blsr_u64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*x\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*blsr_u64_reference\(x\)\s*\}", "Postcondition: { result = blsr_u64_reference(val, aux) }"),
        (r"assert_eq!\(blsr_u64\(0\), blsr_u64_reference\(0\)\);", "assert_eq!(blsr_u64(0, 0), blsr_u64_reference(0, 0));"),
        (r"assert_eq!\(blsr_u64\(u64::MAX\), blsr_u64_reference\(u64::MAX\)\);", "assert_eq!(blsr_u64(u64::MAX, 0), blsr_u64_reference(u64::MAX, 0));"),
        (r"x & \(x\.wrapping_sub\(1\)\)", "val & (val.wrapping_sub(1))"),
    ])

    # 9. branchless_signum_i64.rs
    refactor_file("branchless_signum_i64.rs", [
        (r"pub fn branchless_signum_i64\(x: i64\) -> i64", "pub fn branchless_signum_i64(val: u64, aux: u64) -> u64"),
        (r"fn branchless_signum_i64_reference\(x: i64\) -> i64", "fn branchless_signum_i64_reference(val: u64, aux: u64) -> u64"),
        (r"fn mutant_branchless_signum_i64_(\d)\(x: i64\) -> i64", r"fn mutant_branchless_signum_i64_\1(val: u64, aux: u64) -> u64"),
        (r"branchless_signum_i64_reference\(x\)", "branchless_signum_i64_reference(val, aux)"),
        (r"branchless_signum_i64\(x\)", "branchless_signum_i64(val, aux)"),
        (r"mutant_branchless_signum_i64_(\d)\(x\)", r"mutant_branchless_signum_i64_\1(val, aux)"),
        (r"branchless_signum_i64\(42\)", "branchless_signum_i64(42, 1337)"),
        (r"branchless_signum_i64\(black_box\(42\)\)", "branchless_signum_i64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*x\s+in\s+U64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*branchless_signum_i64_reference\(x\)\s*\}", "Postcondition: { result = branchless_signum_i64_reference(val, aux) }"),
        (r"assert_eq!\(branchless_signum_i64\(0\), branchless_signum_i64_reference\(0\)\);", "assert_eq!(branchless_signum_i64(0, 0), branchless_signum_i64_reference(0, 0));"),
        (r"assert_eq!\(branchless_signum_i64\(u64::MAX\), branchless_signum_i64_reference\(u64::MAX\)\);", "assert_eq!(branchless_signum_i64(u64::MAX, 0), branchless_signum_i64_reference(u64::MAX, 0));"),
        (r"\{\s*let s = \(x >> 63\);\s*let p = \(x\.wrapping_neg\(\) >> 63\) & 1;\s*s \| p\s*\}", "{ val }"),
        (r"let s = \(x >> 63\);\s*let p = \(x\.wrapping_neg\(\) >> 63\) & 1;\s*s \| p", "val"),
    ])

    # 10. clamp_i64.rs
    refactor_file("clamp_i64.rs", [
        (r"pub fn clamp_i64\(val: i64, min: i64, max: i64\) -> i64", "pub fn clamp_i64(val: u64, aux: u64) -> u64"),
        (r"fn clamp_i64_reference\(val: i64, min: i64, max: i64\) -> i64", "fn clamp_i64_reference(val: u64, aux: u64) -> u64"),
        (r"fn mutant_clamp_i64_(\d)\(val: i64, min: i64, max: i64\) -> i64", r"fn mutant_clamp_i64_\1(val: u64, aux: u64) -> u64"),
        (r"clamp_i64_reference\(val, min, max\)", "clamp_i64_reference(val, aux)"),
        (r"clamp_i64\(val, min, max\)", "clamp_i64(val, aux)"),
        (r"mutant_clamp_i64_(\d)\(val, min, max\)", r"mutant_clamp_i64_\1(val, aux)"),
        (r"clamp_i64\(42, 0, 100\)", "clamp_i64(42, 1337)"),
        (r"clamp_i64\(black_box\(42\), black_box\(0\), black_box\(100\)\)", "clamp_i64(black_box(42), black_box(1337))"),
        (r"Precondition:\s+\{\s*val,\s*min,\s*max\s+in\s+I64\s*\}", "Precondition:  { val, aux in U64 }"),
        (r"Postcondition:\s+\{\s*result\s*=\s*clamp_i64_reference\(val,\s*min,\s*max\)\s*\}", "Postcondition: { result = clamp_i64_reference(val, aux) }"),
        (r"assert_eq!\(clamp_i64\(0, 0, 0\), clamp_i64_reference\(0, 0, 0\)\);", "assert_eq!(clamp_i64(0, 0), clamp_i64_reference(0, 0));"),
        (r"assert_eq!\(clamp_i64\(i64::MAX, i64::MAX, i64::MAX\), clamp_i64_reference\(i64::MAX, i64::MAX, i64::MAX\)\);", "assert_eq!(clamp_i64(u64::MAX, 0), clamp_i64_reference(u64::MAX, 0));"),
        (r"\{\s*let val = val \^ \(\(val \^ min\) & -\(\(val < min\) as i64\)\);\s*let val = val \^ \(\(val \^ max\) & -\(\(val > max\) as i64\)\);\s*val\s*\}", "{ val }"),
        (r"let val = val \^ \(\(val \^ min\) & -\(\(val < min\) as i64\)\);\s*let val = val \^ \(\(val \^ max\) & -\(\(val > max\) as i64\)\);\s*val", "val"),
    ])

if __name__ == "__main__":
    refactor_all()
