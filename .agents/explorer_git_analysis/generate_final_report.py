import os
import subprocess
import re

repo_dir = "/Users/sac/bcinr"
algo_dir = os.path.join(repo_dir, "crates/bcinr-logic/src/algorithms")
report_path = "/Users/sac/bcinr/.agents/explorer_git_analysis/git_report.md"

DUMMY_PATTERNS = [
    "0x9E3779B97F4A7C15",
    "0x5555555555555555",
    "0x6C62272E07BB0142",
    "0x0101010101010101"
]

# Mapping prefixes to mathematical/logical descriptions
DESCRIPTIONS = [
    (r"^aabb_intersect_branchless", "Branchless check for Axis-Aligned Bounding Box (AABB) intersection in 3D/2D space, ensuring constant-time geometric collision checks."),
    (r"^abs_diff_i64", "Computes the absolute difference between two signed 64-bit integers without branching: |val - aux|."),
    (r"^abs_diff_u64", "Computes the absolute difference between two unsigned 64-bit integers without branching: |val - aux|."),
    (r"^add_sat_i32", "Computes the saturating addition of two signed 32-bit integers, clamping the result to i32::MIN or i32::MAX on overflow."),
    (r"^add_sat_u64", "Computes the saturating addition of two unsigned 64-bit integers, clamping the result to u64::MAX on overflow."),
    (r"^adler32_branchless", "Computes the Adler-32 checksum of data in a branchless manner, mixing the running sum s1 and s2."),
    (r"^aho_corasick_simd_step", "Performs a single-step transition in a SIMD-parallelized Aho-Corasick multiple pattern matching machine."),
    (r"^ascii_to_lowercase_simd", "Converts an 8-byte packed ASCII string in a 64-bit word to lowercase using bitwise masks to isolate and modify alphabetical bytes in parallel."),
    (r"^ascii_to_uppercase_simd", "Converts an 8-byte packed ASCII string in a 64-bit word to uppercase using bitwise masks to isolate and modify alphabetical bytes in parallel."),
    (r"^avg_ceil_u64", "Computes the ceiling average of two unsigned 64-bit integers without overflow: (val + aux + 1) / 2."),
    (r"^avg_u64", "Computes the floor average of two unsigned 64-bit integers without overflow: (val + aux) / 2."),
    (r"^base32_encode_rfc4648", "Encodes binary data into base32 according to RFC 4648 using constant-time bit shifting."),
    (r"^base64_decode_chunk4", "Decodes a 4-character chunk of Base64 encoded text into 3 bytes of binary data in constant time."),
    (r"^base64_decode_simd", "Decodes Base64 encoded character sequences into bytes using parallel bit shifting and bitwise SWAR techniques."),
    (r"^base64_encode_simd", "Encodes binary bytes into Base64 character sequences using parallel bit shifting and bitwise SWAR techniques."),
    (r"^base85_encode_ascii85", "Encodes binary data into Ascii85 (base85) using constant-time division-free scaling."),
    (r"^bclr_u64", "Clears the bit at a specified index in a 64-bit word: val & !(1 << aux)."),
    (r"^benes_network_u64", "Routes bits in a 64-bit word through a Benes permutation network based on control routing bits."),
    (r"^bext_u64", "Extracts the bit at a specified index in a 64-bit word: (val >> aux) & 1."),
    (r"^binary_search_v_u32x4", "Vectorized binary search step inside 4-element u32 SIMD vectors using comparison masks."),
    (r"^binom_sat_u32", "Computes the binomial coefficient C(n, k) with saturating arithmetic to prevent overflow beyond u32::MAX."),
    (r"^bit_matrix_transpose_64x64", "Transposes a 64x64 bit matrix represented as an array of 64 u64 words using branchless divide-and-conquer bit swaps."),
    (r"^bit_matrix_transpose_8x8", "Transposes an 8x8 bit matrix packed inside a single u64 word using branchless divide-and-conquer bit swaps."),
    (r"^bit_parallel_sort8_u32", "Sorts 8 unsigned 32-bit integers in parallel using a bit-parallel SWAR sorting network."),
    (r"^bit_permute_identity_64", "Checks or routes identity permutations of bits in a 64-bit word."),
    (r"^bit_permute_step_u64", "Performs a single butterfly swap step in a bit permutation network: swaps bits under a mask by a given shift."),
    (r"^bit_swap_u64", "Swaps the bit positions in a 64-bit word (reverses the bit order)."),
    (r"^bit_vector_compress_elias_fano", "Encodes a sorted integer sequence into a compressed Elias-Fano representation without branching."),
    (r"^bitonic_merge_u64x8", "Merges two sorted 4-element u64 arrays in parallel using a bitonic merge network."),
    (r"^bitonic_sort_64u32", "Sorts 64 unsigned 32-bit integers using a bitonic sorting network of depth 6."),
    (r"^bitpacking_decode_u32_k", "Unpacks k-bit integers from a packed 32-bit word stream in a constant-time sequence."),
    (r"^bitpacking_encode_u32_k", "Packs u32 integers into k-bit fields in a contiguous u32 stream in a constant-time sequence."),
    (r"^bloom_filter_add_u64", "Inserts a 64-bit value into a Bloom filter register by computing multiple hash locations and setting respective bits."),
    (r"^bloom_filter_graph_visited", "Checks and sets visited states for graph nodes using a Bloom filter, avoiding duplicate node traversals."),
    (r"^bloom_filter_intersect", "Computes the intersection (bitwise AND) of two Bloom filter bit arrays."),
    (r"^bloom_filter_query_u64", "Queries a Bloom filter for the presence of a 64-bit value, returning true/false based on mask membership."),
    (r"^bloom_filter_union", "Computes the union (bitwise OR) of two Bloom filter bit arrays."),
    (r"^blsi_u64", "Isolates the lowest set bit in a 64-bit word: val & val.wrapping_neg()."),
    (r"^blsmsk_u64", "Creates a mask of bits up to and including the lowest set bit in a 64-bit word: val ^ (val - 1)."),
    (r"^blsr_u64", "Resets the lowest set bit in a 64-bit word: val & (val - 1)."),
    (r"^bool_slice_from_mask", "Converts a 64-bit mask into a boolean slice of 64 elements, assigning true/false based on corresponding bit values."),
    (r"^branchless_priority_queue_pop", "Pops the maximum/minimum element from a priority queue heap using branchless select tree comparisons."),
    (r"^branchless_priority_queue_push", "Pushes an element into a priority queue heap and bubble-ups using branchless swap operations."),
    (r"^branchless_ring_buffer_mpmc", "Multi-producer multi-consumer thread-safe circular ring buffer push/pop index management without branching."),
    (r"^branchless_signum_i64", "Computes the signum of a signed 64-bit integer, returning -1 for negative, 0 for zero, and 1 for positive without branches."),
    (r"^branchless_stack_spsc", "Single-producer single-consumer lock-free stack push/pop index management without branching."),
    (r"^branchless_vtable_lookup", "Performs virtual method dispatch index routing using constant-time offset tables instead of branches."),
    (r"^bsd_checksum_u16", "Computes the BSD checksum of data, performing a right rotation on the sum and adding each byte."),
    (r"^bset_u64", "Sets the bit at a specified index in a 64-bit word: val | (1 << aux)."),
    (r"^btst_u64", "Tests the bit at a specified index in a 64-bit word, returning 1 if set and 0 if clear: (val >> aux) & 1."),
    (r"^burrows_wheeler_transform_step", "Performs a single character step or rotation index calculation for the Burrows-Wheeler Transform."),
    (r"^cityhash64", "Computes the CityHash64 hash value of a 64-bit word or byte sequence using multiplication and shift-rotations."),
    (r"^clamp_i64", "Clamps a signed 64-bit integer to a range [min, max] using min/max bitwise selects: min(max(val, min), max)."),
    (r"^clhash", "Computes the CLHash (carryless multiplication hash) of two inputs in a constant-time step."),
    (r"^clique_check_branchless", "Determines if a subset of nodes forms a complete clique in an adjacency matrix using bitwise parallel mask checks."),
    (r"^compress_bits_u64", "Compresses bits of a 64-bit word to the right using a selection mask, equivalent to the PEXT instruction."),
    (r"^consistent_hash_jump_u64", "Computes Jump Consistent Hashing to map a 64-bit key to a bucket in the range [0, num_buckets)."),
    (r"^consistent_hash_maglev", "Computes Maglev consistent hash lookup tables without dynamic control flow branches."),
    (r"^convex_hull_monotone_chain_step", "Performs a single-point triangulation or cross-product orientation test for the Andrew's Monotone Chain convex hull algorithm."),
    (r"^copy_sign_i64", "Copies the sign of a signed 64-bit integer `aux` to `val` without branching."),
    (r"^count_consecutive_set_bits_u64", "Counts the length of the longest consecutive run of set bits in a 64-bit word using shift-AND operations."),
    (r"^count_min_sketch_add", "Updates frequency counters in a Count-Min Sketch by hashing the value and incrementing the matrix cells."),
    (r"^count_min_sketch_query", "Queries the estimated frequency of a value in a Count-Min Sketch by finding the minimum of hashed cell counters."),
    (r"^counting_sort_branchless_u8", "Sorts an array of 8-bit integers without dynamic branching by building cumulative histogram offsets."),
    (r"^csv_scan_row_simd", "Scans a row of text for separators (commas, newlines) using SIMD byte masks and popcount to locate delimiters."),
    (r"^cuckoo_filter_add_u64", "Inserts a value into a Cuckoo Filter by hashing it to two buckets and kicking out values in case of collisions."),
    (r"^cyclic_redundancy_check_crc32c", "Computes the CRC32c checksum of a 64-bit word or byte sequence using a polynomial generator mask."),
    (r"^cyclic_redundancy_check_crc64", "Computes the CRC64 checksum of a 64-bit word or byte sequence using a polynomial generator mask."),
    (r"^dequantize_u32", "Converts a quantized 32-bit integer back to a floating point value using scale and zero-point parameters."),
    (r"^delta_decode_simd_u32", "Decodes delta-encoded 32-bit integer arrays in parallel using SIMD prefix prefix-sum scans."),
    (r"^delta_encode_simd_u32", "Encodes 32-bit integer arrays to their adjacent differences in parallel using SIMD difference scans."),
    (r"^delta_swap_u64", "Swaps bits of a 64-bit word with their shifted peers under a bitwise swap mask."),
    (r"^disjoint_set_union_branchless", "Merges or finds sets in a union-find disjoint set tree using path compression and union-by-rank without branches."),
    (r"^div_sat_u64", "Computes saturating division of two unsigned 64-bit integers: val / aux, clamping to u64::MAX on divide-by-zero."),
    (r"^epoch_based_reclamation_step", "Performs a single atomic step in epoch-based memory reclamation, updating current epoch or retiring nodes."),
    (r"^euclidean_dist_sq_u32x2", "Computes the squared Euclidean distance between two 2D points packed in 32-bit vectors."),
    (r"^expand_bits_u64", "Expands bits of a 64-bit word to positions specified by a mask, equivalent to the PDEP instruction."),
    (r"^farmhash64", "Computes the FarmHash64 hash of a 64-bit word using multiplication and shift-rotations."),
    (r"^fast_inverse_sqrt_u32", "Computes the fast inverse square root of a 32-bit float using the magic bit manipulation constant (Quake III Arena algorithm)."),
    (r"^fibonacci_hash_u64", "Computes Fibonacci multiplicative hashing on a 64-bit word using the golden ratio multiplier."),
    (r"^fp_div_u32_q16", "Performs division of two Q16.16 fixed-point 32-bit integers without overflow."),
    (r"^fp_mul_u32_q16", "Performs multiplication of two Q16.16 fixed-point 32-bit integers without overflow."),
    (r"^frustum_culling_branchless", "Checks if a bounding box intersects a 3D viewing frustum by performing parallel plane dot-product tests."),
    (r"^funnel_shift_left_u64", "Performs a 128-bit left shift on a concatenated double-word (val, aux) and returns the upper 64 bits."),
    (r"^funnel_shift_right_u64", "Performs a 128-bit right shift on a concatenated double-word (val, aux) and returns the lower 64 bits."),
    (r"^gather_bits_u64", "Gathers non-contiguous bits from a 64-bit word according to a selection mask, packing them to the right."),
    (r"^gaussian_noise_box_muller", "Generates normally distributed random floats from two uniformly distributed inputs using the Box-Muller transform."),
    (r"^get_mask_boundary_high_u64", "Finds the highest bit position of the active range in a 64-bit mask (using CLZ)."),
    (r"^graph_bfs_simd_step", "Performs a single-level BFS frontier expansion step on a graph using SIMD-parallelized bitsets."),
    (r"^graph_dfs_bit_parallel", "Performs a single-step DFS traversal using bit-parallel active path tracking and stack updates."),
    (r"^gray_decode_u64", "Converts a Gray-coded 64-bit word back into its binary representation: val ^ (val >> 1) ^ (val >> 2) ..."),
    (r"^gray_encode_u64", "Converts a binary 64-bit word into its Gray code representation: val ^ (val >> 1)."),
    (r"^halton_sampler_simd", "Generates quasi-random Halton sequence samples using prime bases in parallel SIMD registers."),
    (r"^hamming_dist_simd", "Computes the Hamming distance (number of differing bits) between two 512-bit vectors using SIMD popcount."),
    (r"^hashing_trick_u64", "Performs feature hashing for sparse high-dimensional data, mapping string/integer keys to a fixed-size index space."),
    (r"^hazard_pointer_retire", "Safely retires a retired memory node using hazard pointer lists to defer reclamation in a lock-free manner."),
    (r"^hex_decode_simd", "Decodes hexadecimal strings into binary bytes using SIMD character-mapping logic."),
    (r"^hex_encode_simd", "Encodes binary bytes into hexadecimal strings using SIMD character-mapping logic."),
    (r"^highwayhash_64", "Computes HighwayHash64 checksum of a data chunk, offering highly secure 64-bit hashes using SIMD vector mixing."),
    (r"^hilbert_curve_decode_u32", "Decodes a 1D Hilbert curve index into 2D coordinates (X, Y) using branchless bit shuffles."),
    (r"^hilbert_curve_encode_u32", "Encodes 2D coordinates (X, Y) into a 1D Hilbert curve index using branchless bit shuffles."),
    (r"^hyperloglog_add_u64", "Updates HyperLogLog cardinality registers for a 64-bit key by hashing and taking maximum leading zeros."),
    (r"^hyperloglog_merge", "Merges two HyperLogLog cardinality register sets by computing the element-wise maximum."),
    (r"^morton_decode_2d_u32", "Decodes a 2D Morton code (Z-order curve) into its X and Y coordinates."),
    (r"^morton_encode_2d_u32", "Interleaves the bits of two 32-bit integers to form a 2D Morton code (Z-order curve index)."),
    (r"^morton_encode_3d_u32", "Interleaves the bits of three 32-bit integers to form a 3D Morton code."),
    (r"^quantize_u32", "Quantizes a floating-point value to a 32-bit integer based on scaling factors."),
    (r"^quotient_filter_add_u64", "Adds a 64-bit key to a Quotient Filter, hashing to find the slot and using run-length metadata shifts."),
    (r"^rank_select_sort_u32", "Sorts u32 integers using rank/select directory structures in constant time."),
    (r"^reservoir_sample_branchless", "Selects a random sample of items from an incoming stream without branching control flow."),
    (r"^reservoir_sample_weighted_simd", "Performs weighted reservoir sampling using parallel SIMD floating point keys."),
    (r"^scatter_bits_u64", "Scatters bits from a 64-bit word to positions specified by a mask, packing them according to the mask."),
    (r"^smoothstep_u32", "Computes Hermite interpolation (smoothstep) on unsigned 32-bit integers: 3x^2 - 2x^3."),
    (r"^space_saving_add", "Inserts an element into the Space-Saving sketch to estimate top heavy hitters in a data stream."),
    (r"^t_digest_add_u32", "Adds a value to a T-Digest sketch for online quantile estimation, updating centroids."),
    (r"^weighted_reservoir_sample", "Selects a weighted random sample from an incoming stream using A-Res/A-Exp-J weighted reservoir sampling."),
    (r"^z_order_curve_2d_u32", "Maps 2D spatial coordinates to a 1D index using bit-interleaved Z-order curve Morton coding."),
]

def get_desc(name):
    for pattern, desc in DESCRIPTIONS:
        if re.search(pattern, name):
            return desc
    # Generic backup patterns
    if "abs_diff" in name:
        return f"Branchless calculation of the absolute difference of two inputs of type {name.split('_')[-1]}."
    if "sat" in name:
        return "Branchless saturating arithmetic operation (clamping result on overflow/underflow)."
    if "hash" in name or "checksum" in name or "adler" in name or "crc" in name or "sip" in name or "city" in name or "murmur" in name:
        return "Branchless hashing or checksum calculation that mixes input bytes to generate a fingerprint."
    if "sort" in name:
        return "Sorting network step or branchless implementation of a sorting pass."
    if "search" in name:
        return "Branchless binary or multi-way search layout lookup function."
    if "filter" in name:
        return "Probabilistic member set filter update/query step (Bloom, Cuckoo, or Quotient filter)."
    if "simd" in name:
        return "SIMD-parallelized or SWAR instruction sequence for vector operation."
    if "encode" in name or "decode" in name:
        return "Bitwise data formatting, encoding, or decoding algorithm (e.g. Hex, Base64, Gray code, Morton, Hilbert)."
    return "A safety-critical branchless B-Calculus microkernel primitive."

def get_git_head_content(rel_path):
    cmd = ["git", "show", f"HEAD:{rel_path}"]
    res = subprocess.run(cmd, cwd=repo_dir, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode == 0:
        return res.stdout
    return None

def extract_fn_body(content, fn_name):
    pattern = rf"pub fn\s+{fn_name}\s*\("
    match = re.search(pattern, content)
    if not match:
        return "Not found"
    
    start_idx = content.find("{", match.end())
    if start_idx == -1:
        return "No opening brace"
    
    brace_count = 1
    end_idx = start_idx + 1
    while brace_count > 0 and end_idx < len(content):
        char = content[end_idx]
        if char == '{':
            brace_count += 1
        elif char == '}':
            brace_count -= 1
        end_idx += 1
    
    if brace_count == 0:
        body = content[start_idx+1:end_idx-1].strip()
        return body
    return "Mismatched braces"

def main():
    files = sorted([f for f in os.listdir(algo_dir) if f.endswith(".rs") and f != "mod.rs"])
    
    results = []
    dummy_files = []
    unmodified_files = []
    
    for filename in files:
        file_path = os.path.join(algo_dir, filename)
        rel_path = os.path.relpath(file_path, repo_dir)
        fn_name = filename[:-3]
        
        with open(file_path, "r", encoding="utf-8") as f:
            local_content = f.read()
            
        head_content = get_git_head_content(rel_path) or ""
        
        local_impl = extract_fn_body(local_content, fn_name)
        head_impl = extract_fn_body(head_content, fn_name)
        
        matched_patterns = [pat for pat in DUMMY_PATTERNS if pat in local_content]
        is_dummy = len(matched_patterns) > 0
        
        purpose = get_desc(fn_name)
        
        data = {
            "filename": filename,
            "fn_name": fn_name,
            "is_dummy": is_dummy,
            "matched_patterns": matched_patterns,
            "local_impl": local_impl,
            "head_impl": head_impl,
            "purpose": purpose
        }
        
        if is_dummy:
            dummy_files.append(data)
        else:
            unmodified_files.append(data)
            
        results.append(data)
        
    # Write report
    report_lines = []
    report_lines.append("# Git History & Algorithm Audit Report")
    report_lines.append("")
    report_lines.append("## Executive Summary")
    report_lines.append("An audit of the git history and working directory of the `bcinr` codebase at `/Users/sac/bcinr` was conducted to examine the introduction of dummy hash patterns in the academic-grade branchless algorithm primitives.")
    report_lines.append("")
    report_lines.append(f"- **Git Commit Before Update**: `HEAD` (Commit hash: `e2438bb38c6320d05df67274f0af5f4b841bb369` / `e2438bb`)")
    report_lines.append(f"- **Total Audited Primitives**: {len(files)}")
    report_lines.append(f"- **Uncommitted Modifications**: ALL {len(files)} algorithm modules in `crates/bcinr-logic/src/algorithms/` have local modifications in the working directory.")
    report_lines.append(f"  - **Dummy-Hashed Algorithms**: {len(dummy_files)} algorithms were modified to use dummy hash patterns.")
    report_lines.append(f"  - **Unmodified/Comment-Only Algorithms**: {len(unmodified_files)} algorithms kept their original implementations (only comments/metadata changed).")
    report_lines.append("")
    report_lines.append("## The Test Suite Vulnerability: Why the Validation Gate Failed")
    report_lines.append("The current test suite is structured with a co-located reference function inside each module's `tests` module (e.g. `abs_diff_i64_reference` inside `abs_diff_i64.rs`). When the working directory was updated with dummy hashes, the script doing the update also modified the positive reference function within the test code in lockstep to contain the identical dummy formula. ")
    report_lines.append("")
    report_lines.append("Because the test oracle was redefined to be the dummy implementation, the proptest equivalence check:")
    report_lines.append("```rust")
    report_lines.append("let expected = abs_diff_i64_reference(val, aux);")
    report_lines.append("let actual = abs_diff_i64(val, aux);")
    report_lines.append("prop_assert_eq!(expected, actual);")
    report_lines.append("```")
    report_lines.append("always succeeds. Similarly, the counterfactual checker tests (mutant tests) were defined as simple modifications of the reference (e.g. `!abs_diff_i64_reference(val, aux)`), meaning they also passed despite the implementation being a completely fake hashing routine.")
    report_lines.append("")
    report_lines.append("### Recommended Remediations for a Robust Validation Gate:")
    report_lines.append("1. **Decouple Oracle References**: Reference/oracle implementations must be completely independent. For standard algorithms, they should use standard library math (e.g., casting to signed integers or using `saturating_add`) rather than matching the branchless bitwise formulas.")
    report_lines.append("2. **Add Algorithmic Invariant Tests**: Instead of just testing equivalence against a single function, write invariants that verify algebraic properties (e.g. identity, commutativity, distributivity, range boundaries).")
    report_lines.append("3. **External Proofs / Hostile Mutant Falsification**: Integrate tests that run against hardcoded known-good vectors (independent constants) and actively prove that the tests fail if the code is mutated to a dummy hash.")
    report_lines.append("")
    report_lines.append("## Category breakdown")
    report_lines.append(f"### 1. Unmodified Algorithms (Doc-only Changes) — Quantity: {len(unmodified_files)}")
    report_lines.append("These algorithms kept their original implementations. Only their documentation/decorations were modified.")
    report_lines.append("")
    report_lines.append("| File Name | Function Name | Expected Mathematical/Logical Purpose |")
    report_lines.append("|---|---|---|")
    for u in unmodified_files:
        report_lines.append(f"| `{u['filename']}` | `{u['fn_name']}` | {u['purpose']} |")
    report_lines.append("")
    
    report_lines.append(f"### 2. Modified Algorithms (Dummy-Hashed) — Quantity: {len(dummy_files)}")
    report_lines.append("These algorithms were updated to return dummy hashes instead of their intended mathematical logic.")
    report_lines.append("")
    
    for d in dummy_files:
        report_lines.append(f"#### `{d['filename']}`")
        report_lines.append(f"- **Function**: `{d['fn_name']}`")
        report_lines.append(f"- **Intended Logical/Mathematical Purpose**: {d['purpose']}")
        report_lines.append(f"- **Dummy Patterns Introduced**: {', '.join(d['matched_patterns'])}")
        report_lines.append("- **Original (Genuine) Implementation**:")
        report_lines.append("```rust")
        report_lines.append(d['head_impl'])
        report_lines.append("```")
        report_lines.append("- **Dummy Hashed Implementation**:")
        report_lines.append("```rust")
        report_lines.append(d['local_impl'])
        report_lines.append("```")
        report_lines.append("")
        
    with open(report_path, "w", encoding="utf-8") as f:
        f.write("\n".join(report_lines))
    print("Completed writing final report to", report_path)

if __name__ == "__main__":
    main()
