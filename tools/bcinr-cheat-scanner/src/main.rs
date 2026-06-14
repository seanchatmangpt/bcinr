use std::fs;
use std::path::Path;
use std::process;
use walkdir::WalkDir;

fn main() {
    let root = "crates/bcinr-logic/src/algorithms";
    let mut total_findings = 0;

    for entry in WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
    {
        let path = entry.path();
        match fs::read_to_string(path) {
            Ok(src) => {
                total_findings += pass_cancel_xor(&src, path);
                total_findings += pass_circular_ref(&src, path);
                total_findings += pass_magic_const(&src, path);
                total_findings += pass_padding(&src, path);
                total_findings += pass_fake_proof(&src, path);
            }
            Err(e) => {
                eprintln!("ERROR: Failed to read {}: {}", path.display(), e);
            }
        }
    }

    if total_findings > 0 {
        eprintln!(
            "\n{} cheat finding(s). Fix before committing.",
            total_findings
        );
        process::exit(1);
    }

    println!(
        "OK: no cheat patterns detected across {} algorithm files.",
        count_algos(root)
    );
}

fn count_algos(root: &str) -> usize {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        .count()
}

/// Pass 1: Detect self-canceling XOR patterns like A.wrapping_add(B) ^ A
fn pass_cancel_xor(src: &str, path: &Path) -> usize {
    let mut findings = 0;
    for (line_no, line) in src.lines().enumerate() {
        if detect_self_cancel(line) {
            eprintln!(
                "CHEAT[CANCEL_XOR]: {}:{} — self-canceling expression detected",
                path.display(),
                line_no + 1
            );
            findings += 1;
        }
    }
    findings
}

fn detect_self_cancel(line: &str) -> bool {
    // Pattern: a line that contains X.wrapping_add(...) ^ X (or variants)
    // Find ` ^ ` and check if what follows is a duplicate of what preceded wrapping_add
    if let Some(xor_pos) = line.find(" ^ ") {
        let rhs = line[xor_pos + 3..].trim_end();
        let lhs = &line[..xor_pos];

        // Look for .wrapping_add in lhs; extract the expression before it
        if let Some(wa_pos) = lhs.rfind(".wrapping_add(") {
            let before_wa = lhs[..wa_pos].trim();

            // Check if RHS ends with the same expression (stripping trailing parens)
            let rhs_normalized = rhs.trim_end_matches(')').trim();

            // Simple heuristic: if the normalized RHS matches what's before wrapping_add
            if before_wa == rhs_normalized || rhs_normalized.ends_with(before_wa) {
                return true;
            }

            // Also catch the pattern where the entire expression is duplicated
            // e.g., "((X) ^ Y).rotate_left(5)) ^ ((X) ^ Y).rotate_left(5))"
            // by looking for any ')' followed by ' ^ ' followed by the same tail again
            let pattern = format!("{}) ^ ({})", before_wa, before_wa);
            if line.contains(&pattern) {
                return true;
            }
        }
    }

    false
}

/// Pass 2: Detect circular references where _reference == implementation
fn pass_circular_ref(src: &str, path: &Path) -> usize {
    let mut findings = 0;

    // Quick heuristic: parse the file and extract function bodies
    // We'll use a simple text-based approach to avoid heavy parsing

    // Look for "fn <name>(val: u64, aux: u64) -> u64 {" and "fn <name>_reference(val: u64, aux: u64) -> u64 {"
    // Extract the body between the braces for both

    let lines: Vec<&str> = src.lines().collect();

    for i in 0..lines.len() {
        let line = lines[i];
        if line.contains("pub fn") && line.contains("(val: u64, aux: u64) -> u64") {
            // Found a public function; extract its name
            if let Some(name_start) = line.find("pub fn ") {
                let after_fn = &line[name_start + 7..];
                if let Some(paren_pos) = after_fn.find('(') {
                    let func_name = after_fn[..paren_pos].trim();

                    // Now look for the reference function in test module
                    let ref_func_name = format!("{}_reference", func_name);
                    for j in i..lines.len() {
                        let ref_line = lines[j];
                        if ref_line.contains(&format!("fn {}", ref_func_name)) {
                            // Extract bodies
                            if let (Some(pub_body), Some(ref_body)) =
                                (extract_fn_body(&lines, i), extract_fn_body(&lines, j))
                            {
                                if normalize_code(&pub_body) == normalize_code(&ref_body) {
                                    eprintln!(
                                        "CHEAT[CIRCULAR_REF]: {} — {} _reference body identical to implementation; tests prove nothing",
                                        path.display(), func_name
                                    );
                                    findings += 1;
                                }
                            }
                            break;
                        }
                    }
                }
            }
        }
    }

    findings
}

fn extract_fn_body(lines: &[&str], start_idx: usize) -> Option<String> {
    if start_idx >= lines.len() {
        return None;
    }

    let line = lines[start_idx];
    if let Some(brace_pos) = line.find('{') {
        let mut body = String::new();
        let mut brace_count = 1;
        let after_first_brace = &line[brace_pos + 1..];
        body.push_str(after_first_brace);

        for line_text in &lines[(start_idx + 1)..] {
            body.push('\n');
            body.push_str(line_text);
            for ch in line_text.chars() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            // Remove the final '}'
                            return Some(body[..body.len() - 1].to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    None
}

fn normalize_code(code: &str) -> String {
    // Strip whitespace, collapse multiple spaces, normalize line endings
    code.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .collect::<String>()
}

/// Pass 3: Detect magic constants in production code (not in test modules)
fn pass_magic_const(src: &str, path: &Path) -> usize {
    let mut findings = 0;
    let magic_values = ["0xDEADBEEF", "0xCAFEBABE", "0xdeadbeef", "0xcafebabe"];

    let mut in_test_module = false;
    let mut test_depth = 0;

    for (line_no, line) in src.lines().enumerate() {
        // Track #[cfg(test)] modules
        if line.contains("#[cfg(test)]") {
            in_test_module = true;
            test_depth = 1;
            continue;
        }

        if in_test_module {
            // Count braces to track when we exit the test module
            for ch in line.chars() {
                match ch {
                    '{' => test_depth += 1,
                    '}' => {
                        test_depth -= 1;
                        if test_depth == 0 {
                            in_test_module = false;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Check for magic constants outside test modules
        if !in_test_module {
            for magic in &magic_values {
                if line.contains(magic) {
                    eprintln!(
                        "CHEAT[MAGIC_CONST]: {}:{} — forbidden magic constant {} in production code",
                        path.display(),
                        line_no + 1,
                        magic
                    );
                    findings += 1;
                    break; // Only report once per line
                }
            }
        }
    }

    findings
}

/// Pass 4: Detect padding boilerplate (file-length inflation)
fn pass_padding(src: &str, path: &Path) -> usize {
    let mut findings = 0;

    // Look for the sentinel line
    if src.contains("PADDING ENSURING FILE LENGTH REQUIREMENT") {
        eprintln!(
            "CHEAT[PADDING]: {} — artificial file-length inflation detected",
            path.display()
        );
        findings += 1;
    }

    // Also check for numbered padding lines (// N. Line N pattern)
    let mut consecutive_padding = 0;
    for line in src.lines() {
        if is_numbered_padding_line(line) {
            consecutive_padding += 1;
            if consecutive_padding >= 5 {
                // Report only once if we find 5+ consecutive
                if consecutive_padding == 5 {
                    eprintln!(
                        "CHEAT[PADDING]: {} — numbered padding block detected",
                        path.display()
                    );
                    findings += 1;
                }
            }
        } else {
            consecutive_padding = 0;
        }
    }

    findings
}

fn is_numbered_padding_line(line: &str) -> bool {
    // Match lines like "// 1. Line 1", "// 2. Line 2", etc.
    let trimmed = line.trim();
    if !trimmed.starts_with("//") {
        return false;
    }
    let after_slashes = trimmed[2..].trim();
    // Check if it matches the pattern "N. Line N"
    let parts: Vec<&str> = after_slashes.split('.').collect();
    if parts.len() >= 2 {
        if let Ok(num) = parts[0].trim().parse::<u32>() {
            return (1..=32).contains(&num) && parts[1].trim().starts_with("Line");
        }
    }
    false
}

/// Pass 5: Detect fake Hoare-logic proof comments
fn pass_fake_proof(src: &str, path: &Path) -> usize {
    let mut findings = 0;
    let mut hoare_count = 0;

    for line in src.lines() {
        if line.contains("Hoare-logic Verification Line")
            && line.contains("Branchless path is the unique solution to the state constraints of")
        {
            hoare_count += 1;
        }
    }

    if hoare_count >= 5 {
        eprintln!(
            "CHEAT[FAKE_PROOF]: {} — {} boilerplate Hoare-logic lines detected (not real proofs)",
            path.display(),
            hoare_count
        );
        findings += 1;
    }

    findings
}
