import os
import re

algorithms_dir = "crates/bcinr-logic/src/algorithms"
rust_file = "test_fuzz_all.rs"

files = [f for f in os.listdir(algorithms_dir) if f.endswith(".rs") and f != "mod.rs"]

rust_code = """
fn next_u64(rng: &mut u64) -> u64 {
    *rng ^= *rng << 13;
    *rng ^= *rng >> 7;
    *rng ^= *rng << 17;
    *rng
}

fn main() {
    println!("Starting fuzzer...");
}
"""

with open(rust_file, "w") as f:
    f.write(rust_code)
