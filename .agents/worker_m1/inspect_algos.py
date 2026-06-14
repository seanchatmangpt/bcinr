import os
import re

ALGO_DIR = "/Users/sac/bcinr/crates/bcinr-logic/src/algorithms"

def inspect():
    files = sorted(f for f in os.listdir(ALGO_DIR) if f.endswith(".rs") and f != "mod.rs")
    print(f"Total files: {len(files)}")
    
    sig_pat = re.compile(r"pub fn\s+(\w+)\s*\(([^)]*)\)\s*->\s*(\w+)")
    
    non_standard = []
    for f in files:
        path = os.path.join(ALGO_DIR, f)
        content = open(path).read()
        match = sig_pat.search(content)
        if match:
            fn_name, params, ret_type = match.groups()
            params_clean = params.strip().replace("\n", "").replace(" ", "")
            if params_clean != "val:u64,aux:u64" or ret_type != "u64":
                non_standard.append((f, fn_name, params, ret_type))
        else:
            print(f"No match for {f}")
            
    print(f"Non-standard functions: {len(non_standard)}")
    for f, fn_name, params, ret_type in non_standard:
        print(f"{f}: pub fn {fn_name}({params}) -> {ret_type}")

if __name__ == "__main__":
    inspect()
