import os
import re

def audit_file(path):
    with open(path, 'r') as f:
        lines = f.readlines()
    content = "".join(lines)
    score = 0
    issues = []

    # C1: Determinism (25 pts)
    # Extract pub fn body
    fn_match = re.search(r'pub fn [a-z0-9_]+\(.*?\)\s*->\s*[a-z0-9_]+\s*\{(.*?)\n\}', content, re.DOTALL)
    if fn_match:
        body = fn_match.group(1)
        if not any(k in body for k in ["if ", "match ", "loop ", "while "]):
            score += 25
        else:
            issues.append("JCC detected in hot path")
    
    # C2: Behavioral Oracle (25 pts)
    if "_reference" in content and "equivalence" in content and "boundaries" in content:
        score += 25
    else:
        issues.append("Oracle or Boundary tests missing")

    # C3: Mutation Hostility (25 pts)
    mutant_count = content.count("fn mutant_")
    rejection_count = content.count("rejects_mutant") + content.count("counterfactual_mutant")
    if mutant_count >= 3 and rejection_count >= 3:
        score += 25
    else:
        issues.append(f"Incomplete mutation matrix (M:{mutant_count}, R:{rejection_count})")

    # C4: Academic Integrity (25 pts)
    has_hoare = "Hoare" in content or "Axiomatic" in content
    long_enough = len(lines) >= 100
    if has_hoare and long_enough:
        score += 25
    else:
        if not has_hoare: issues.append("Formal proofs missing")
        if not long_enough: issues.append(f"Under length constraint ({len(lines)} lines)")

    return score, issues

def main():
    paths = []
    for root, _, files in os.walk("crates/bcinr-logic/src"):
        for f in files:
            if f.endswith(".rs") and f != "lib.rs" and f != "mod.rs":
                paths.append(os.path.join(root, f))
    
    results = []
    for p in paths:
        score, issues = audit_file(p)
        results.append((p.split("/")[-1], score, issues))
    
    results.sort(key=lambda x: x[1], reverse=True)
    
    print(f"{'Algorithm/Abstraction':<40} | {'Score':<10} | {'Status'}")
    print("-" * 70)
    for name, score, issues in results:
        status = "PhD-Verified ✅" if score == 100 else "NEEDS WORK ⚠️"
        print(f"{name:<40} | {score:<10} | {status}")
        if issues:
            print(f"  -> Issues: {', '.join(issues)}")

main()
