import os

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

def write_bench(filename, subset):
    with open(filename, "w") as f:
        f.write("use bcinr_logic::algorithms::*;\n")
        f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\n\n")
        
        bench_name = filename.split("/")[-1].replace(".rs", "")
        f.write(f"fn {bench_name}(c: &mut Criterion) {{\n")
        for algo in subset:
            f.write(f"    // Import each module explicitly if needed or use the star import\n")
            f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\n")
            f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}(black_box(42), black_box(1337))));\n')
            f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}(black_box(0), black_box(0))));\n')
            f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}(black_box(u64::MAX), black_box(u64::MAX))));\n')
        f.write("}\n\n")
        
        f.write(f"criterion_group!(benches, {bench_name});\n")
        f.write("criterion_main!(benches);\n")

# Split into chunks of 100
for i in range(0, len(ALGORITHMS), 100):
    subset = ALGORITHMS[i:i+100]
    filename = f"bcinr-bench/benches/algorithms_{i+1}_{min(i+100, len(ALGORITHMS))}.rs"
    write_bench(filename, subset)

# Also update all_300_bench.rs
with open("bcinr-bench/benches/all_300_bench.rs", "w") as f:
    f.write("use bcinr_logic::algorithms::*;\n")
    f.write("use criterion::{criterion_group, criterion_main, Criterion, black_box};\n\n")
    
    for algo in ALGORITHMS:
        f.write(f"fn bench_{algo}(c: &mut Criterion) {{\n")
        f.write(f"    use bcinr_logic::algorithms::{algo}::{algo};\n")
        f.write(f'    c.bench_function("{algo}_avg", |b| b.iter(|| {algo}(black_box(42), black_box(1337))));\n')
        f.write(f'    c.bench_function("{algo}_min", |b| b.iter(|| {algo}(black_box(0), black_box(0))));\n')
        f.write(f'    c.bench_function("{algo}_max", |b| b.iter(|| {algo}(black_box(u64::MAX), black_box(u64::MAX))));\n')
        f.write("}\n\n")
    
    f.write("criterion_group!(benches,\n")
    for algo in ALGORITHMS:
        f.write(f"    bench_{algo},\n")
    f.write(");\n")
    f.write("criterion_main!(benches);\n")
