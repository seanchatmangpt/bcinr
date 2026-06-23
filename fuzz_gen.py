import os
import re
import subprocess

algorithms_dir = "crates/bcinr-logic/src/algorithms"
test_file = "test_all_fuzz.rs"

files = [f for f in os.listdir(algorithms_dir) if f.endswith(".rs") and f != "mod.rs"]

imports = []
tests = []

for f in files:
    path = os.path.join(algorithms_dir, f)
    with open(path, "r") as file:
        content = file.read()
    
    # Check if it has a reference function
    ref_match = re.search(r'fn ([a-zA-Z0-9_]+_reference)\((.*?)\)\s*->\s*(.*?)\s*\{', content)
    if not ref_match:
        continue
        
    func_match = re.search(r'pub fn ([a-zA-Z0-9_]+)\((.*?)\)\s*->\s*(.*?)\s*\{', content)
    if not func_match:
        continue
        
    ref_name = ref_match.group(1)
    func_name = func_match.group(1)
    args = func_match.group(2)
    
    # We only handle (val: u64, aux: u64) -> u64 for now
    if "val: u64, aux: u64" in args or "aux: u64, val: u64" in args:
        # Extract the reference implementation to copy it into the test file
        # We need to extract the whole block
        # Actually, simpler to just run proptest natively or append a #[test] to the file?
        pass

# It might be easier to just inject a proptest into every file that has a reference but lacks a proptest!
