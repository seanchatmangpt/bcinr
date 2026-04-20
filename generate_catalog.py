import os
import re

def extract_info(path):
    with open(path, 'r') as f:
        content = f.read()
    
    # Extract name
    name = os.path.basename(path).replace(".rs", "")
    
    # Extract first paragraph of doc comment
    doc_match = re.search(r'/// (.*?)\n///\s*\n', content, re.DOTALL)
    desc = doc_match.group(1).replace("/// ", "").strip() if doc_match else "Branchless primitive."

    # Extract usage doctest
    usage_match = re.search(r'/// ```rust(.*?)\n/// ```', content, re.DOTALL)
    usage = usage_match.group(1).replace("///", "").strip() if usage_match else "// No usage example."
    
    return name, desc, usage

def main():
    catalog = {
        "1. Advanced Bit Manipulation": [],
        "2. Numerical & Saturation Calculus": [],
        "3. Sorting Networks & Permutations": [],
        "4. Deterministic Hashing & Sketching": [],
        "5. Branchless Parsing & Encoding": [],
        "6. Graph & Geometry Substrates": [],
        "7. Execution Substrate Abstractions": []
    }

    # Algorithms
    algo_files = sorted([f for f in os.listdir("crates/bcinr-logic/src/algorithms") if f.endswith(".rs") and f != "mod.rs"])
    for i, f in enumerate(algo_files):
        path = os.path.join("crates/bcinr-logic/src/algorithms", f)
        name, desc, usage = extract_info(path)
        
        # Categorize by index (approximate based on implementation order)
        idx = i + 1
        if idx <= 50: key = "1. Advanced Bit Manipulation"
        elif idx <= 100: key = "2. Numerical & Saturation Calculus"
        elif idx <= 150: key = "3. Sorting Networks & Permutations"
        elif idx <= 200: key = "4. Deterministic Hashing & Sketching"
        elif idx <= 250: key = "5. Branchless Parsing & Encoding"
        else: key = "6. Graph & Geometry Substrates"
        
        catalog[key].append((name, desc, usage))

    # Abstractions
    abst_files = sorted([f for f in os.listdir("crates/bcinr-logic/src/abstractions") if f.endswith(".rs") and f != "mod.rs"])
    for f in abst_files:
        path = os.path.join("crates/bcinr-logic/src/abstractions", f)
        name, desc, usage = extract_info(path)
        catalog["7. Execution Substrate Abstractions"].append((name, desc, usage))

    with open("ALGORITHM_CATALOG.md", "w") as f:
        f.write("# BCINR Algorithm & Abstraction Catalog\n\n")
        f.write("This catalog documents the entire 306-module branchless substrate, grouped by operational family.\n\n")
        
        for family, items in catalog.items():
            f.write(f"## {family}\n\n")
            for name, desc, usage in items:
                f.write(f"### `{name}`\n")
                f.write(f"**Description:** {desc}\n\n")
                f.write("**Usage:**\n")
                f.write(f"```rust\n{usage}\n```\n\n")
                f.write("---\n\n")

main()
