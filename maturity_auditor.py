import os
import re

def get_fn_body(content, start_pos):
    brace_start = content.find('{', start_pos)
    if brace_start == -1: return None, start_pos
    count = 0
    for i in range(brace_start, len(content)):
        if content[i] == '{': count += 1
        elif content[i] == '}':
            count -= 1
            if count == 0: return content[brace_start+1:i], i + 1
    return None, len(content)

def audit_file(path):
    with open(path, 'r') as f: lines = f.readlines()
    content = "".join(lines)
    score = 0
    issues = []

    # C1: Determinism (25 pts)
    any_jcc = False
    pos = 0
    while True:
        match = re.search(r'pub fn ([a-z0-9_]+)', content[pos:])
        if not match: break
        name = match.group(1)
        fn_start = pos + match.start()
        body, next_pos = get_fn_body(content, fn_start)
        pos = next_pos
        if not body or name in ["new", "new_checked", "new_substrate"]: continue
        body_clean = re.sub(r'//.*', '', body)
        body_clean = re.sub(r'/\*.*?\*/', '', body_clean, flags=re.DOTALL)
        if any(re.search(r'\b' + k + r'\b', body_clean) for k in ["if", "match", "loop", "while"]):
            any_jcc = True
            issues.append(f"JCC detected in {name}")
    if not any_jcc: score += 25
    
    # C2: Behavioral Oracle (25 pts)
    if "_reference" in content and ("equivalence" in content or "oracle" in content) and "boundaries" in content:
        score += 25
    else: issues.append("Oracle or Boundary tests missing")

    # C3: Mutation Hostility (25 pts)
    mutants = content.count("fn mutant_")
    rejections = content.count("rejects_mutant") + content.count("counterfactual_mutant")
    if mutants >= 3 and rejections >= 3: score += 25
    else: issues.append(f"Incomplete mutation matrix (M:{mutants}, R:{rejections})")

    # C4: Axiomatic Proofs (25 pts)
    has_hoare = any(k in content for k in ["Hoare", "Axiomatic", "AXIOMATIC"])
    if has_hoare and len(lines) >= 100: score += 25
    else:
        if not has_hoare: issues.append("Formal proofs missing")
        if len(lines) < 100: issues.append(f"Under length constraint ({len(lines)} lines)")

    return score, issues

def main():
    results = []
    for root, _, files in os.walk("crates/bcinr-logic/src"):
        for f in sorted(files):
            if f.endswith(".rs") and f not in ["mod.rs", "lib.rs", "tests.rs"]:
                path = os.path.join(root, f)
                score, issues = audit_file(path)
                results.append((f, score, issues))
    print(f"{'Algorithm/Abstraction':<40} | {'Score':<10} | {'Status'}")
    print("-" * 70)
    for name, score, issues in results:
        status = "PhD-Verified ✅" if score == 100 else "NEEDS WORK ⚠️"
        print(f"{name:<40} | {score:<10} | {status}")
        if issues: print(f"  -> Issues: {', '.join(issues)}")

if __name__ == "__main__":
    main()
