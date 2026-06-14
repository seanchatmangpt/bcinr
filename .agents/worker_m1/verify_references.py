import os
import re
from pathlib import Path

ALGO_DIR = Path("/Users/sac/bcinr/crates/bcinr-logic/src/algorithms")

# Import ORACLES and category_for logic directly from u64_audit
import sys
sys.path.append("/Users/sac/bcinr/tools")
from u64_audit import category_for, ORACLES

def check_all():
    files = sorted(p for p in ALGO_DIR.glob("*.rs") if p.name != "mod.rs")
    bad_files = []
    
    REFERENCE_FN_RE = re.compile(
        r"fn\s+([a-zA-Z0-9_]+)_reference\(val:\s*u64,\s*aux:\s*u64\)\s*->\s*u64\s*\{(.*?)\}",
        re.DOTALL
    )
    
    for p in files:
        cat = category_for(p.stem)
        expected_oracle = ORACLES[cat].strip().replace("\n", "").replace(" ", "")
        content = p.read_text()
        
        match = REFERENCE_FN_RE.search(content)
        if not match:
            print(f"Error: reference function not found/matched in {p.name}")
            bad_files.append((p.name, "missing_reference_fn"))
            continue
            
        fn_name, body = match.groups()
        body_clean = body.strip().replace("\n", "").replace(" ", "")
        if body_clean != expected_oracle:
            print(f"Mismatch in {p.name}:")
            print(f"  Found:    {body_clean[:50]}...")
            print(f"  Expected: {expected_oracle[:50]}...")
            bad_files.append((p.name, "mismatch"))
            
    print(f"Checked {len(files)} files. Bad: {len(bad_files)}")

if __name__ == "__main__":
    check_all()
