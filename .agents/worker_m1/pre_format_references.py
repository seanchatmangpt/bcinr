import os
import re
from pathlib import Path

ALGO_DIR = Path("/Users/sac/bcinr/crates/bcinr-logic/src/algorithms")

def pre_format_all():
    files = sorted(p for p in ALGO_DIR.glob("*.rs") if p.name != "mod.rs")
    
    count = 0
    for p in files:
        content = p.read_text()
        
        # Match fn name_reference(args) -> ret {
        match = re.search(r"fn\s+([a-zA-Z0-9_]+)_reference\s*\((.*?)\)\s*->\s*(\w+)\s*\{", content)
        if not match:
            print(f"Warning: reference function start not found in {p.name}")
            continue
            
        name = match.group(1)
        start_idx = match.start()
        
        # Find matching closing brace
        brace_count = 0
        end_idx = -1
        for i in range(start_idx, len(content)):
            if content[i] == '{':
                brace_count += 1
            elif content[i] == '}':
                brace_count -= 1
                if brace_count == 0:
                    end_idx = i + 1
                    break
                    
        if end_idx == -1:
            print(f"Brace mismatch in {p.name}")
            continue
            
        # Replace the entire reference function block with a clean placeholder format
        new_ref_fn = f"    fn {name}_reference(val: u64, aux: u64) -> u64 {{\n        // placeholder\n    }}\n"
        
        new_content = content[:start_idx] + new_ref_fn + content[end_idx:]
        p.write_text(new_content)
        count += 1
        
    print(f"Successfully pre-formatted reference functions in {count}/{len(files)} files.")

if __name__ == "__main__":
    pre_format_all()
