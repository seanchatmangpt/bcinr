import os
import re

BENCH_DIR = "/Users/sac/bcinr/bcinr-bench/benches"

def refactor_file(filename):
    path = os.path.join(BENCH_DIR, filename)
    if not os.path.exists(path):
        print(f"Error: {path} not found")
        return
    content = open(path).read()

    # blsi_u64
    content = re.sub(r"blsi_u64\(black_box\((\w+(?:::\w+)?)\)\)", r"blsi_u64(black_box(\1), black_box(1337))", content)
    # blsmsk_u64
    content = re.sub(r"blsmsk_u64\(black_box\((\w+(?:::\w+)?)\)\)", r"blsmsk_u64(black_box(\1), black_box(1337))", content)
    # blsr_u64
    content = re.sub(r"blsr_u64\(black_box\((\w+(?:::\w+)?)\)\)", r"blsr_u64(black_box(\1), black_box(1337))", content)
    # branchless_signum_i64
    content = re.sub(r"branchless_signum_i64\(black_box\(42i64\)\)", "branchless_signum_i64(black_box(42u64), black_box(1337))", content)
    content = re.sub(r"branchless_signum_i64\(black_box\(0i64\)\)", "branchless_signum_i64(black_box(0u64), black_box(1337))", content)
    content = re.sub(r"branchless_signum_i64\(black_box\(i64::MAX\)\)", "branchless_signum_i64(black_box(u64::MAX), black_box(1337))", content)
    # clamp_i64
    content = re.sub(r"clamp_i64\(black_box\(42i64\),\s*black_box\(42i64\),\s*black_box\(42i64\)\)", "clamp_i64(black_box(42u64), black_box(1337))", content)
    content = re.sub(r"clamp_i64\(black_box\(0i64\),\s*black_box\(0i64\),\s*black_box\(0i64\)\)", "clamp_i64(black_box(0u64), black_box(1337))", content)
    content = re.sub(r"clamp_i64\(black_box\(i64::MAX\),\s*black_box\(i64::MAX\),\s*black_box\(i64::MAX\)\)", "clamp_i64(black_box(u64::MAX), black_box(1337))", content)

    with open(path, "w") as f:
        f.write(content)
    print(f"Refactored {filename}")

if __name__ == "__main__":
    refactor_file("algorithms_1_100.rs")
    refactor_file("all_300_bench.rs")
