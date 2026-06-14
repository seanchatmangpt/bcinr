import os
import subprocess
import re

# Directory paths
repo_dir = "/Users/sac/bcinr"
algo_dir = os.path.join(repo_dir, "crates/bcinr-logic/src/algorithms")

# Dummy constants
DUMMY_PATTERNS = [
    "0x9E3779B97F4A7C15",
    "0x5555555555555555",
    "0x6C62272E07BB0142",
    "0x0101010101010101"
]

def get_git_head_content(rel_path):
    cmd = ["git", "show", f"HEAD:{rel_path}"]
    res = subprocess.run(cmd, cwd=repo_dir, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
    if res.returncode == 0:
        return res.stdout
    return None

def extract_fn_body(content, fn_name):
    # Find pub fn {fn_name}
    pattern = rf"pub fn\s+{fn_name}\s*\("
    match = re.search(pattern, content)
    if not match:
        return "Not found"
    
    # Let's find the first '{' after the match
    start_idx = content.find("{", match.end())
    if start_idx == -1:
        return "No opening brace"
    
    # Brace tracking to find the matching '}'
    brace_count = 1
    end_idx = start_idx + 1
    while brace_count > 0 and end_idx < len(content):
        char = content[end_idx]
        if char == '{':
            brace_count += 1
        elif char == '}':
            brace_count -= 1
        end_idx += 1
    
    if brace_count == 0:
        body = content[start_idx+1:end_idx-1].strip()
        return body
    return "Mismatched braces"

def main():
    files = sorted([f for f in os.listdir(algo_dir) if f.endswith(".rs") and f != "mod.rs"])
    
    report_lines = []
    report_lines.append("# Git History & Algorithm Audit Report")
    report_lines.append("")
    report_lines.append(f"Total algorithm modules audited: {len(files)}")
    report_lines.append("")
    
    # We will categorize files
    dummy_count = 0
    modified_count = 0
    unique_count = 0
    
    results = []
    
    for filename in files:
        file_path = os.path.join(algo_dir, filename)
        rel_path = os.path.relpath(file_path, repo_dir)
        fn_name = filename[:-3]
        
        # Read working copy
        with open(file_path, "r", encoding="utf-8") as f:
            local_content = f.read()
            
        # Read HEAD copy
        head_content = get_git_head_content(rel_path)
        
        if head_content is None:
            # New file in working copy or error
            head_content = ""
            
        # Extract implementations
        local_impl = extract_fn_body(local_content, fn_name)
        head_impl = extract_fn_body(head_content, fn_name)
        
        # Check if local contains dummy patterns
        matched_patterns = [pat for pat in DUMMY_PATTERNS if pat in local_content]
        is_dummy = len(matched_patterns) > 0
        
        if is_dummy:
            dummy_count += 1
        
        # Check if modified from HEAD
        is_modified = local_content != head_content
        if is_modified:
            modified_count += 1
            
        # Unique implementations in HEAD (not dummy hash oracle)
        matched_head_patterns = [pat for pat in DUMMY_PATTERNS if pat in head_content]
        is_head_dummy = len(matched_head_patterns) > 0
        if not is_head_dummy:
            unique_count += 1
            
        # Derive mathematical logic / purpose
        # Let's try to parse from doc comments in the file
        purpose = "TBD"
        doc_match = re.search(r"///\s+(.*)\n///", local_content)
        if doc_match:
            purpose = doc_match.group(1).strip()
        else:
            doc_lines = []
            for line in local_content.splitlines():
                if line.strip().startswith("///"):
                    doc_lines.append(line.replace("///", "").strip())
                elif doc_lines and not line.strip().startswith("///"):
                    break
            if doc_lines:
                purpose = " ".join(doc_lines[:2])
                
        results.append({
            "filename": filename,
            "fn_name": fn_name,
            "is_dummy": is_dummy,
            "matched_patterns": matched_patterns,
            "is_modified": is_modified,
            "local_impl": local_impl,
            "head_impl": head_impl,
            "purpose": purpose
        })
        
    report_lines.append("## Summary Statistics")
    report_lines.append(f"- **Total Algorithm Files**: {len(files)}")
    report_lines.append(f"- **Files modified in working copy**: {modified_count}")
    report_lines.append(f"- **Files containing dummy hashes in working copy**: {dummy_count}")
    report_lines.append(f"- **Files with unique implementations in HEAD (pre-modification)**: {unique_count}")
    report_lines.append("")
    report_lines.append("## Detailed File Listing")
    report_lines.append("")
    
    for r in results:
        report_lines.append(f"### `{r['filename']}`")
        report_lines.append(f"- **Function**: `{r['fn_name']}`")
        report_lines.append(f"- **Purpose**: {r['purpose']}")
        report_lines.append(f"- **Contains Dummy Patterns (Working Copy)**: {'Yes (' + ', '.join(r['matched_patterns']) + ')' if r['is_dummy'] else 'No'}")
        report_lines.append(f"- **Modified from HEAD**: {'Yes' if r['is_modified'] else 'No'}")
        report_lines.append("- **Original Implementation (HEAD)**:")
        report_lines.append("```rust")
        report_lines.append(r['head_impl'])
        report_lines.append("```")
        report_lines.append("- **Current Implementation (Working Copy)**:")
        report_lines.append("```rust")
        report_lines.append(r['local_impl'])
        report_lines.append("```")
        report_lines.append("")
        
    report_content = "\n".join(report_lines)
    
    report_path = "/Users/sac/bcinr/.agents/explorer_git_analysis/git_report.md"
    with open(report_path, "w", encoding="utf-8") as f:
        f.write(report_content)
    print("Report written successfully to", report_path)

if __name__ == "__main__":
    main()
