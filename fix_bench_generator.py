import os
import re

def get_signature(algo_name):
    file_path = f"crates/bcinr-logic/src/algorithms/{algo_name}.rs"
    if not os.path.exists(file_path):
        return None
    with open(file_path, 'r') as f:
        content = f.read()
    
    # Look for pub fn algo_name( ... )
    pattern = rf'pub fn {algo_name}\s*\((.*?)\)'
    match = re.search(pattern, content, re.DOTALL)
    if match:
        args_str = match.group(1).strip()
        if not args_str:
            return []
        
        # Split by comma, but be careful with nested types if any (usually not)
        args = []
        for arg in args_str.split(','):
            arg = arg.strip()
            if not arg: continue
            # arg is "name: type"
            parts = arg.split(':')
            if len(parts) == 2:
                args.append(parts[1].strip())
        return args
    return None

def get_all_algos():
    with open("crates/bcinr-logic/src/algorithms/mod.rs", "r") as f:
        content = f.read()
    algos = []
    for line in content.splitlines():
        if line.startswith("pub mod "):
            algo = line.split("pub mod ")[1].strip(";").strip()
            algos.append(algo)
    return algos

ALGORITHMS = get_all_algos()
signatures = {}
for algo in ALGORITHMS:
    sig = get_signature(algo)
    if sig is not None:
        signatures[algo] = sig

# Now write the new generate_benchmarks.py
with open("generate_benchmarks_fixed.py", "w") as f:
    f.write("import os\n\n")
    f.write(f"SIGNATURES = {signatures!r}\n\n")
    f.write("ALGORITHMS = sorted(SIGNATURES.keys())\n\n")
    f.write("""
def write_bench(filename, subset):
    with open(filename, "w") as f:
        f.write("use bcinr_logic::algorithms::*;\\n")
        f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\\n\\n")
        
        bench_name = filename.split("/")[-1].replace(".rs", "")
        f.write(f"fn {bench_name}(c: &mut Criterion) {{\\n")
        for algo in subset:
            types = SIGNATURES[algo]
            def get_val(t, val):
                if t == 'i64': return f"{val}i64"
                if t == 'i32': return f"{val}i32"
                if t == 'u32': return f"{val}u32"
                if t == 'u128': return f"{val}u128"
                if t == 'f64': return f"{val}.0f64"
                if t == 'f32': return f"{val}.0f32"
                return str(val)

            def get_max(t):
                if t == 'i64': return "i64::MAX"
                if t == 'i32': return "i32::MAX"
                if t == 'u32': return "u32::MAX"
                if t == 'u64': return "u64::MAX"
                if t == 'u128': return "u128::MAX"
                return "100" # fallback

            args_avg = ", ".join([f"black_box({get_val(t, 42)})" for t in types])
            args_min = ", ".join([f"black_box({get_val(t, 0)})" for t in types])
            args_max = ", ".join([f"black_box({get_max(t)})" for t in types])

            f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\\n")
            f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}({args_avg})));\\n')
            f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}({args_min})));\\n')
            f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}({args_max})));\\n')
        f.write("}\\n\\n")
        
        f.write(f"criterion_group!(benches, {bench_name});\\n")
        f.write("criterion_main!(benches);\\n")

# Split into chunks of 100
for i in range(0, len(ALGORITHMS), 100):
    subset = ALGORITHMS[i:i+100]
    filename = f"bcinr-bench/benches/algorithms_{i+1}_{min(i+100, len(ALGORITHMS))}.rs"
    write_bench(filename, subset)

# Also update all_300_bench.rs
with open("bcinr-bench/benches/all_300_bench.rs", "w") as f:
    f.write("use bcinr_logic::algorithms::*;\\n")
    f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\\n\\n")
    
    for algo in ALGORITHMS:
        f.write(f"fn bench_{algo}(c: &mut Criterion) {{\\n")
        f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\\n")
        types = SIGNATURES[algo]
        def get_val(t, val):
            if t == 'i64': return f"{val}i64"
            if t == 'i32': return f"{val}i32"
            if t == 'u32': return f"{val}u32"
            if t == 'u128': return f"{val}u128"
            if t == 'f64': return f"{val}.0f64"
            if t == 'f32': return f"{val}.0f32"
            return str(val)

        def get_max(t):
            if t == 'i64': return "i64::MAX"
            if t == 'i32': return "i32::MAX"
            if t == 'u32': return "u32::MAX"
            if t == 'u64': return "u64::MAX"
            if t == 'u128': return "u128::MAX"
            return "100" # fallback

        args_avg = ", ".join([f"black_box({get_val(t, 42)})" for t in types])
        args_min = ", ".join([f"black_box({get_val(t, 0)})" for t in types])
        args_max = ", ".join([f"black_box({get_max(t)})" for t in types])

        f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}({args_avg})));\\n')
        f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}({args_min})));\\n')
        f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}({args_max})));\\n')
        f.write("}\\n\\n")
    
    f.write("criterion_group!(benches,\\n")
    for algo in ALGORITHMS:
        f.write(f"    bench_{algo},\\n")
    f.write(");\\n")
    f.write("criterion_main!(benches);\\n")
""")
