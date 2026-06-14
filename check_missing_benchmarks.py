import os
import sys

def main():
    # Get all implemented algorithms
    algo_dir = "crates/bcinr-logic/src/algorithms"
    algos = [f[:-3] for f in os.listdir(algo_dir) if f.endswith(".rs") and f != "mod.rs"]
    
    # Get all benchmarked algorithms
    bench_file = "bcinr-bench/benches/all_300_bench.rs"
    if not os.path.exists(bench_file):
        print(f"FAILED: {bench_file} missing.")
        sys.exit(1)
        
    with open(bench_file, "r") as f:
        content = f.read()
        
    missing = []
    for algo in algos:
        if f'"{algo}_avg"' not in content:
            missing.append(algo)
            
    if not missing:
        print(f"SUCCESS: All {len(algos)} algorithms are benchmarked.")
    else:
        print(f"FAILED: Found {len(missing)} missing benchmarks:")
        for m in missing:
            print(f"  - {m}")
        sys.exit(1)

if __name__ == "__main__":
    main()
