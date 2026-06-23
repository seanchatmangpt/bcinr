import os
import re

algorithms_dir = "crates/bcinr-logic/src/algorithms"

files = [f for f in os.listdir(algorithms_dir) if f.endswith(".rs") and f != "mod.rs"]

for f in files:
    path = os.path.join(algorithms_dir, f)
    with open(path, "r") as file:
        lines = file.readlines()
        
    new_lines = []
    bench_mod = False
    imports = set()
    
    for i, line in enumerate(lines):
        if "pub mod bench {" in line:
            bench_mod = True
            imports = set()
            new_lines.append(line)
        elif bench_mod and "use alloc::vec::Vec;" in line:
            if "Vec" not in imports:
                imports.add("Vec")
                new_lines.append(line)
        else:
            new_lines.append(line)
            
    with open(path, "w") as file:
        file.writelines(new_lines)

print("Cleaned duplicate Vec imports")
