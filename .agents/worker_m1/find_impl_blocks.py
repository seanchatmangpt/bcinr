import os
import re
from pathlib import Path

ALGO_DIR = Path("/Users/sac/bcinr/crates/bcinr-logic/src/algorithms")

def check():
    files = sorted(p for p in ALGO_DIR.glob("*.rs") if p.name != "mod.rs")
    
    IMPL_RE = re.compile(
        r"pub fn\s+([a-zA-Z0-9_]+)\(val:\s*u64,\s*aux:\s*u64\)\s*->\s*u64\s*\{(.*?)\}\s*\n\n#\[cfg\(test\)\]",
        re.DOTALL
    )
    
    unmatched = []
    for p in files:
        content = p.read_text()
        match = IMPL_RE.search(content)
        if not match:
            unmatched.append(p.name)
            
    print(f"Total: {len(files)}, Unmatched: {len(unmatched)}")
    if unmatched:
        print("First 10 unmatched:", unmatched[:10])

if __name__ == "__main__":
    check()
