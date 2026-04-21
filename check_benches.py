import subprocess
import re

def main():
    print("Running benchmarks...")
    process = subprocess.Popen(
        ['cargo', 'bench', '-p', 'bcinr-bench', '--bench', 'all_300_bench', '--features', 'alloc', '--color', 'never', '--', '--sample-size', '10', '--noplot', '--measurement-time', '1'],
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True
    )

    current_bench = None
    over_threshold = []
    
    # Regex to match the criterion output line:
    # time:   [894.43 ps 898.31 ps 902.94 ps]
    # We want the middle value
    time_re = re.compile(r'time:\s+\[[^\]]+\s+([\d\.]+)\s+(ps|ns|µs|ms|s)\s+[^\]]+\]')

    for line in process.stdout:
        print(line, end="")
        if line.startswith("Benchmarking "):
            current_bench = line.strip().split(" ")[1]
        
        match = time_re.search(line)
        if match and current_bench:
            val = float(match.group(1))
            unit = match.group(2)
            
            # Convert to ns
            val_ns = val
            if unit == 'ps':
                val_ns = val / 1000.0
            elif unit == 'ns':
                val_ns = val
            elif unit == 'µs':
                val_ns = val * 1000.0
            elif unit == 'ms':
                val_ns = val * 1000000.0
            elif unit == 's':
                val_ns = val * 1000000000.0
                
            if val_ns > 100.0:
                over_threshold.append((current_bench, val_ns, f"{val} {unit}"))

    process.wait()
    
    print("\n" + "="*50)
    if not over_threshold:
        print("SUCCESS: All benchmarks are under 100 ns!")
    else:
        print(f"FAILED: {len(over_threshold)} benchmarks exceed 100 ns threshold:")
        for bench, val_ns, orig in over_threshold:
            print(f"  - {bench}: {orig} ({val_ns:.2f} ns)")

if __name__ == "__main__":
    main()
