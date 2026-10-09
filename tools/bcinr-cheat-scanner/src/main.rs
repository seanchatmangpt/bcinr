use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use syn::visit::{self, Visit};
use syn::{BinOp, Expr, ImplItemFn, ItemFn};
use walkdir::WalkDir;

#[derive(Clone)]
pub struct CheatRule {
    pub id: String,
    pub title: String,
    pub constitutional_clause: String,
    pub severity: String,
    pub layers: Vec<String>,
    pub authoritative_only: bool,
    pub detection_contract: String,
    pub required_fixture_ids: Vec<String>,
    pub required_mutant_ids: Vec<String>,
    pub remediation_code: String,
}

fn get_rules() -> Vec<CheatRule> {
    vec![
        CheatRule {
            id: "CHEAT-001".to_string(),
            title: "SELF_CANCELING_OPERATIONS".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-001)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string(), "Text".to_string()],
            authoritative_only: true,
            detection_contract: "Detects expressions that cancel themselves out (e.g. A ^ A, A - A, or A.wrapping_add(B) ^ A)".to_string(),
            required_fixture_ids: vec!["fixture_cancel_xor".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Remove self-canceling terms or simplify the formula.".to_string(),
        },
        CheatRule {
            id: "CHEAT-002".to_string(),
            title: "CIRCULAR_ORACLE".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-002)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: false,
            detection_contract: "Reference oracle function body identical to production implementation body.".to_string(),
            required_fixture_ids: vec!["fixture_circular_oracle".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Implement distinct oracles (e.g. floating point, SMT, or mathematical formulas).".to_string(),
        },
        CheatRule {
            id: "CHEAT-003".to_string(),
            title: "MAGIC_CONSTANTS".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-003)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Text".to_string(), "AST".to_string()],
            authoritative_only: true,
            detection_contract: "Forbidden magic literals (e.g. 0xDEADBEEF, 0xCAFEBABE) controlling production behavior.".to_string(),
            required_fixture_ids: vec!["fixture_magic_const".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Use named, derived, or certified configuration constants instead of unexplained literals.".to_string(),
        },
        CheatRule {
            id: "CHEAT-004".to_string(),
            title: "ARTIFICIAL_FILE_INFLATION".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-004)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Text".to_string()],
            authoritative_only: false,
            detection_contract: "Detects sentinel comments or consecutive numbered lines used for padding file length.".to_string(),
            required_fixture_ids: vec!["fixture_padding".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Remove redundant comments or artificial line inflation.".to_string(),
        },
        CheatRule {
            id: "CHEAT-005".to_string(),
            title: "BOILERPLATE_VERIFICATION_CLAIMS".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-005)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Text".to_string()],
            authoritative_only: false,
            detection_contract: "Detects repetitive boiler-plate verification comments without real proof references.".to_string(),
            required_fixture_ids: vec!["fixture_fake_proof".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Provide real axiomatic proofs or remove the mock comments.".to_string(),
        },
        CheatRule {
            id: "CHEAT-006".to_string(),
            title: "SCANNER_EVASION".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-006)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string(), "Text".to_string()],
            authoritative_only: true,
            detection_contract: "Detects obfuscated operators, macro-nested control flow, or evading formatting.".to_string(),
            required_fixture_ids: vec!["fixture_scanner_evasion".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Avoid hiding branching/complexity in macros; express code directly and cleanly.".to_string(),
        },
        CheatRule {
            id: "CHEAT-007".to_string(),
            title: "DEAD_PATH_COMPLIANCE".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-007)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: true,
            detection_contract: "Detects dead or unreachable code blocks displaying compliance while active path is not.".to_string(),
            required_fixture_ids: vec!["fixture_dead_path".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Remove dead paths and make the active hot path fully compliant.".to_string(),
        },
        CheatRule {
            id: "CHEAT-008".to_string(),
            title: "BENCHMARK_THEATER".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-008)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: false,
            detection_contract: "Detects benchmarks that call functions but do not consume outputs via black_box.".to_string(),
            required_fixture_ids: vec!["fixture_bench_theater".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Feed benchmark outputs into core::hint::black_box to prevent compiler optimization.".to_string(),
        },
        CheatRule {
            id: "CHEAT-009".to_string(),
            title: "MUTANT_THEATER".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-009)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: false,
            detection_contract: "Detects mutants that are trivial or not verified by assertions of typed refusals.".to_string(),
            required_fixture_ids: vec!["fixture_mutant_theater".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Strengthen assertions in counterfactual tests to verify exact typed failure codes.".to_string(),
        },
        CheatRule {
            id: "CHEAT-010".to_string(),
            title: "GATE_JURISDICTION_THEATER".to_string(),
            constitutional_clause: "Rule 16 (Anti-cheat manifesto: CHEAT-010)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Text".to_string()],
            authoritative_only: false,
            detection_contract: "Check if the scanner itself omits target directories/crates from its scan paths.".to_string(),
            required_fixture_ids: vec!["fixture_jurisdiction_theater".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Ensure target paths in bcinr-cheat-scanner include all production crates.".to_string(),
        },
        CheatRule {
            id: "CHEAT-014".to_string(),
            title: "REACHABLE_DEPENDENCY_BRANCH".to_string(),
            constitutional_clause: "Rule 7 (Whole-call-graph branchlessness)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Call-graph".to_string(), "Object-code".to_string()],
            authoritative_only: true,
            detection_contract: "Transitive dependencies containing branches in reachable symbols.".to_string(),
            required_fixture_ids: vec!["fixture_dep_branch".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Audit or rewrite transitive dependencies to ensure they are fully branchless.".to_string(),
        },
        CheatRule {
            id: "CHEAT-020".to_string(),
            title: "MUTATION_BEFORE_ADMISSION".to_string(),
            constitutional_clause: "Rule 10 (No mutation before complete admission)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["AST".to_string()],
            authoritative_only: true,
            detection_contract: "Mutating persistent state fields before completing admission verification checks.".to_string(),
            required_fixture_ids: vec!["fixture_mutation_before_admission".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Perform all validations first and assign values to state at the end of the transaction.".to_string(),
        },
        CheatRule {
            id: "CHEAT-021".to_string(),
            title: "REJECTION_STATE_DRIFT".to_string(),
            constitutional_clause: "Rule 10, Rule 18 (Typed refusals)".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Behavioral hostile".to_string()],
            authoritative_only: true,
            detection_contract: "Rejection of operations causing state modifications (State Drift).".to_string(),
            required_fixture_ids: vec!["fixture_state_drift".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Ensure transaction commits are completely masked and rejected paths leave state unchanged.".to_string(),
        },
        CheatRule {
            id: "CHEAT-031".to_string(),
            title: "BLACK_BOX_BRANCHLESSNESS_CLAIM".to_string(),
            constitutional_clause: "Rule 3, Rule 7".to_string(),
            severity: "ERROR".to_string(),
            layers: vec!["Text".to_string()],
            authoritative_only: false,
            detection_contract: "Documentation claiming core::hint::black_box guarantees machine-level branchlessness.".to_string(),
            required_fixture_ids: vec!["fixture_black_box_claim".to_string()],
            required_mutant_ids: vec![],
            remediation_code: "Remove claims asserting black_box guarantees branchless assembly; rely on object-code disasm audits instead.".to_string(),
        },
    ]
}

impl CheatRule {
    // Dummy field for remediation
    pub fn reremediation_code(&self) -> &str {
        &self.remediation_code
    }
}

struct SynCheatVisitor<'a> {
    path: &'a Path,
    findings: Vec<String>,
    functions: Vec<(String, String)>, // Name and stringified body
}

fn is_simple_expr(e: &Expr) -> bool {
    match e {
        Expr::Path(_) => true,
        Expr::Field(f) => is_simple_expr(&f.base),
        Expr::Index(idx) => is_simple_expr(&idx.expr) && is_simple_expr(&idx.index),
        Expr::Lit(_) => true,
        Expr::Reference(r) => is_simple_expr(&r.expr),
        _ => false,
    }
}

impl<'ast> Visit<'ast> for SynCheatVisitor<'_> {
    fn visit_expr(&mut self, i: &'ast Expr) {
        // CHEAT-001: SELF_CANCELING_OPERATIONS
        if let Expr::Binary(b) = i {
            if matches!(b.op, BinOp::BitXor(_) | BinOp::Sub(_)) {
                let left = &b.left;
                let right = &b.right;
                let left_str = quote::quote!(#left).to_string().replace(" ", "");
                let right_str = quote::quote!(#right).to_string().replace(" ", "");
                if left_str == right_str {
                    self.findings.push(format!(
                        "CHEAT[CHEAT-001]: {} — self-canceling expression detected: {} ^/sub {}",
                        self.path.display(),
                        left_str,
                        right_str
                    ));
                }

                // Check (A.wrapping_add(B)) ^ A
                if let Expr::MethodCall(mc) = &*b.left {
                    if (mc.method == "wrapping_add" || mc.method == "wrapping_sub")
                        && is_simple_expr(&mc.receiver)
                    {
                        let receiver = &mc.receiver;
                        let rec_str = quote::quote!(#receiver).to_string().replace(" ", "");
                        if rec_str == right_str {
                            self.findings.push(format!(
                                "CHEAT[CHEAT-001]: {} — self-canceling expression detected: {} .wrapping_add/sub(...) ^ {}",
                                self.path.display(),
                                rec_str,
                                right_str
                            ));
                        }
                    }
                }
                // Check A ^ (A.wrapping_add(B))
                if let Expr::MethodCall(mc) = &*b.right {
                    if (mc.method == "wrapping_add" || mc.method == "wrapping_sub")
                        && is_simple_expr(&mc.receiver)
                    {
                        let receiver = &mc.receiver;
                        let rec_str = quote::quote!(#receiver).to_string().replace(" ", "");
                        if rec_str == left_str {
                            self.findings.push(format!(
                                "CHEAT[CHEAT-001]: {} — self-canceling expression detected: {} ^ {} .wrapping_add/sub(...)",
                                self.path.display(),
                                left_str,
                                rec_str
                            ));
                        }
                    }
                }
            }
        }

        // CHEAT-003: MAGIC_CONSTANTS (AST check for numeric literals)
        if let Expr::Lit(l) = i {
            if let syn::Lit::Int(li) = &l.lit {
                if let Ok(val) = li.base10_parse::<u64>() {
                    if val == 3735928559 || val == 3405691582 {
                        self.findings.push(format!(
                            "CHEAT[CHEAT-003]: {} — magic constant literal detected: 0x{:X}",
                            self.path.display(),
                            val
                        ));
                    }
                }
            }
        }

        // CHEAT-008: BENCHMARK_THEATER (AST check for criterion bench calling functions without black_box)
        if let Expr::MethodCall(mc) = i {
            if mc.method == "bench_function" || mc.method == "iter" {
                let arg_str = quote::quote!(#mc).to_string();
                // If it is calling algorithms in the benchmark but missing black_box
                if (arg_str.contains("branchless") || arg_str.contains("allocate"))
                    && !arg_str.contains("black_box")
                {
                    self.findings.push(format!(
                        "CHEAT[CHEAT-008]: {} — benchmark theater: return value of branchless call not consumed via black_box",
                        self.path.display()
                    ));
                }
            }
        }

        visit::visit_expr(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast syn::ItemMod) {
        let is_test = i.attrs.iter().any(|attr| {
            let attr_str = quote::quote!(#attr).to_string();
            attr_str.contains("test") || attr_str.contains("bench")
        });
        if is_test {
            return;
        }
        visit::visit_item_mod(self, i);
    }

    fn visit_item_impl(&mut self, i: &'ast syn::ItemImpl) {
        let is_test = i.attrs.iter().any(|attr| {
            let attr_str = quote::quote!(#attr).to_string();
            attr_str.contains("test") || attr_str.contains("bench")
        });
        if is_test {
            return;
        }
        visit::visit_item_impl(self, i);
    }

    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let is_test = i.attrs.iter().any(|attr| {
            let attr_str = quote::quote!(#attr).to_string();
            attr_str.contains("test") || attr_str.contains("bench")
        });
        if is_test {
            return;
        }
        let name = i.sig.ident.to_string();
        let block = &i.block;
        let body_str = quote::quote!(#block).to_string().replace(" ", "");
        self.functions.push((name, body_str));
        visit::visit_item_fn(self, i);
    }

    fn visit_impl_item_fn(&mut self, i: &'ast ImplItemFn) {
        let is_test = i.attrs.iter().any(|attr| {
            let attr_str = quote::quote!(#attr).to_string();
            attr_str.contains("test") || attr_str.contains("bench")
        });
        if is_test {
            return;
        }
        let name = i.sig.ident.to_string();
        let block = &i.block;
        let body_str = quote::quote!(#block).to_string().replace(" ", "");
        self.functions.push((name, body_str));
        visit::visit_impl_item_fn(self, i);
    }

    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
        // CHEAT-006: SCANNER_EVASION
        if let Some(ident) = &i.mac.path.get_ident() {
            if *ident == "macro_rules" {
                let mac_str = quote::quote!(#i).to_string();
                if has_token(&mac_str, "if") || has_token(&mac_str, "match") {
                    self.findings.push(format!(
                        "CHEAT[CHEAT-006]: {} — macro hides control flow or branches (scanner evasion)",
                        self.path.display()
                    ));
                }
            }
        }
        visit::visit_item_macro(self, i);
    }
}

fn has_token(text: &str, token: &str) -> bool {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .any(|t| t == token)
}

fn check_circular_oracles(functions: &[(String, String)], path: &Path, findings: &mut Vec<String>) {
    // CHEAT-002: CIRCULAR_ORACLE
    for (name, body) in functions {
        if name.ends_with("_reference") || name.ends_with("_oracle") {
            let base_name = name
                .trim_end_matches("_reference")
                .trim_end_matches("_oracle");
            for (p_name, p_body) in functions {
                if p_name == base_name && body == p_body {
                    findings.push(format!(
                        "CHEAT[CHEAT-002]: {} — circular oracle: {} identical to implementation {}",
                        path.display(),
                        name,
                        p_name
                    ));
                }
            }
        }
    }
}

fn check_mutation_before_admission(
    functions: &[(String, String)],
    path: &Path,
    findings: &mut Vec<String>,
) {
    // CHEAT-020: MUTATION_BEFORE_ADMISSION
    for (name, body) in functions {
        if name == "allocate" {
            // Check if there is speculative state modification before validation checks
            if body.contains("weights[")
                && body.find("weights[").unwrap() < body.find("const_lt_u32").unwrap_or(usize::MAX)
            {
                findings.push(format!(
                    "CHEAT[CHEAT-020]: {} — mutation before admission in {}",
                    path.display(),
                    name
                ));
            }
        }
    }
}

fn scan_file_text_rules(src: &str, path: &Path, findings: &mut Vec<String>) {
    let normalized = src.to_lowercase();
    let is_test =
        path.to_string_lossy().contains("/tests/") || path.to_string_lossy().contains("/benches/");

    // CHEAT-003: MAGIC_CONSTANTS (Doc comment or text scan)
    if !is_test {
        let mut in_test_module = false;
        let mut test_depth = 0;
        for line in src.lines() {
            let line_lower = line.to_lowercase();
            if line_lower.contains("#[cfg(test)]") || line_lower.contains("mod tests") {
                in_test_module = true;
                test_depth = 1;
            }
            if in_test_module {
                for ch in line.chars() {
                    if ch == '{' {
                        test_depth += 1;
                    } else if ch == '}' {
                        test_depth -= 1;
                        if test_depth == 0 {
                            in_test_module = false;
                        }
                    }
                }
                continue;
            }

            let text_no_us = line.replace("_", "").to_lowercase();
            if text_no_us.contains("0xdeadbeef") || text_no_us.contains("0xcafebabe") {
                findings.push(format!(
                    "CHEAT[CHEAT-003]: {} — magic constant found in file text/doc comments",
                    path.display()
                ));
                break;
            }
        }
    }

    // CHEAT-004: ARTIFICIAL_FILE_INFLATION
    if src.contains("PADDING ENSURING FILE LENGTH REQUIREMENT") {
        findings.push(format!(
            "CHEAT[CHEAT-004]: {} — artificial file-length inflation detected",
            path.display()
        ));
    }
    let mut consecutive_padding = 0;
    for line in src.lines() {
        if line.trim().starts_with("//") {
            let after_slashes = line.trim()[2..].trim();
            if after_slashes.contains(". Line") {
                consecutive_padding += 1;
                if consecutive_padding >= 5 {
                    findings.push(format!(
                        "CHEAT[CHEAT-004]: {} — numbered padding block detected",
                        path.display()
                    ));
                    break;
                }
            }
        } else {
            consecutive_padding = 0;
        }
    }

    // CHEAT-005: BOILERPLATE_VERIFICATION_CLAIMS
    let mut hoare_count = 0;
    for line in src.lines() {
        if line.contains("Hoare-logic Verification Line")
            && line.contains("Branchless path is the unique solution to the state constraints of")
        {
            hoare_count += 1;
            if hoare_count >= 5 {
                findings.push(format!(
                    "CHEAT[CHEAT-005]: {} — mock Hoare-logic verification claims detected",
                    path.display()
                ));
                break;
            }
        }
    }

    // CHEAT-007: DEAD_PATH_COMPLIANCE
    if src.contains("if false {") && src.contains("dummy_branchless") {
        findings.push(format!(
            "CHEAT[CHEAT-007]: {} — dead-path compliance: compliant dummy placed in unreachable if-false block",
            path.display()
        ));
    }

    // CHEAT-009: MUTANT_THEATER
    // §16 targets detection ONLY by inequality: a counterfactual test whose
    // sole assertion surface is assert_ne!. A file that asserts exact typed
    // variants (assert_eq! on Enum::Variant / Err(Path::..)) and uses
    // assert_ne! only as a secondary contrast witnessing that the typed
    // assertions discriminate is the anti-vacuity pattern, not theater.
    // Line comments are stripped first: naming the pattern in prose is not
    // using it.
    if is_test && src.contains("mutant") {
        let code: String = src
            .lines()
            .map(|l| match l.find("//") {
                Some(i) => &l[..i],
                None => l,
            })
            .collect::<Vec<&str>>()
            .join("\n");
        let uses_inequality_detection = code.contains("assert_ne!(");
        let has_typed_surface = window_contains(&code, "assert_eq!", "::", 120)
            || window_contains(&code, "Err(", "::", 80)
            || window_contains(&code, "Ok(", "::", 80);
        if uses_inequality_detection && !has_typed_surface {
            findings.push(format!(
                "CHEAT[CHEAT-009]: {} — mutant theater: test uses weak assert_ne instead of asserting a typed refusal",
                path.display()
            ));
        }
    }

    // CHEAT-021: REJECTION_STATE_DRIFT
    // We expect tests to check that state variables remain bit-for-bit unchanged on rejection.
    // If a test folder contains case_studies, verify that test_rejection_invariance or similar check exists.
    if path.to_string_lossy().contains("/tests/")
        && path.to_string_lossy().contains("case_studies.rs")
        && !src.contains("test_rejection_invariance")
    {
        findings.push(format!(
            "CHEAT[CHEAT-021]: {} — rejection state drift: case studies missing test_rejection_invariance check",
            path.display()
        ));
    }

    // CHEAT-031: BLACK_BOX_BRANCHLESSNESS_CLAIM
    if normalized.contains("black_box guarantees")
        || normalized.contains("black_box ensures branchlessness")
    {
        findings.push(format!(
            "CHEAT[CHEAT-031]: {} — invalid claim: black_box does not guarantee LLVM branchlessness",
            path.display()
        ));
    }
}

fn workspace_crate_roots() -> Vec<PathBuf> {
    // Jurisdiction is derived, never hardcoded (CHEAT-010 remediation): every
    // workspace member crate under crates/ is in scope. Falls back to
    // enumerating crates/*/Cargo.toml when cargo metadata is unavailable.
    // An empty return is refused fail-closed in main: a green gate with empty
    // jurisdiction is not evidence.
    let mut roots = Vec::new();
    let metadata = std::process::Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output();
    if let Ok(out) = metadata {
        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
            if let Some(packages) = json["packages"].as_array() {
                for pkg in packages {
                    if let Some(mp) = pkg["manifest_path"].as_str() {
                        let manifest = PathBuf::from(mp);
                        if let Some(crate_dir) = manifest.parent() {
                            if crate_dir.parent().is_some_and(|p| p.ends_with("crates")) {
                                roots.push(crate_dir.to_path_buf());
                            }
                        }
                    }
                }
            }
        }
    }
    if roots.is_empty() {
        if let Ok(entries) = fs::read_dir("crates") {
            for e in entries.flatten() {
                if e.path().join("Cargo.toml").is_file() {
                    roots.push(e.path());
                }
            }
        }
    }
    roots.sort();
    roots
}

fn check_gate_jurisdiction_theater(roots: &[PathBuf], findings: &mut Vec<String>) {
    // CHEAT-010: GATE_JURISDICTION_THEATER. Unlike the previous string-grep
    // self-check (which could never fire while the two root strings existed
    // anywhere in this file), this check is falsifiable: any crates/* crate
    // directory with a Cargo.toml that the derived roots omit is a finding.
    let root_names: Vec<&str> = roots
        .iter()
        .filter_map(|r| r.file_name().and_then(|n| n.to_str()))
        .collect();
    if let Ok(entries) = fs::read_dir("crates") {
        for e in entries.flatten() {
            let p = e.path();
            if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                if p.join("Cargo.toml").is_file() && !root_names.contains(&name) {
                    findings.push(format!(
                        "CHEAT[CHEAT-010]: {} — crate directory present but absent from scanner roots",
                        p.display()
                    ));
                }
            }
        }
    }
}

fn scan_dependencies(_findings: &mut Vec<String>) {
    // CHEAT-014: REACHABLE_DEPENDENCY_BRANCH. This scanner performs metadata
    // discovery only; the rule's declared object-code layer (branch audits of
    // transitive dependency symbols) is NOT implemented here. Reported honestly
    // on every run from main() so the gap cannot hide behind a green exit.
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output();
    if let Ok(out) = output {
        let json: serde_json::Value = match serde_json::from_slice(&out.stdout) {
            Ok(j) => j,
            Err(_) => return,
        };
        if let Some(packages) = json["packages"].as_array() {
            for pkg in packages {
                let name = pkg["name"].as_str().unwrap_or("");
                let is_workspace_member = pkg["source"].as_str().is_none();
                // If it is not a workspace member and is a dependency of bcinr crates
                if !is_workspace_member && (name == "proptest" || name == "criterion") {
                    // Check if it contains any conditional branches in production code if reachable
                    // Here we can just mock-scan or do simple checks. Let's do a basic scan if source dir exists.
                    if let Some(manifest_path) = pkg["manifest_path"].as_str() {
                        if let Some(parent) = Path::new(manifest_path).parent() {
                            let src_dir = parent.join("src");
                            if src_dir.exists() {
                                // Scan one source file to verify
                                for entry in
                                    WalkDir::new(&src_dir).into_iter().filter_map(|e| e.ok())
                                {
                                    if entry.path().extension().is_some_and(|ext| ext == "rs") {
                                        // Just confirm we checked it
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let _rules = get_rules();
    let roots = workspace_crate_roots();
    let mut findings = Vec::new();
    let mut total_files = 0;

    check_gate_jurisdiction_theater(&roots, &mut findings);

    if roots.is_empty() {
        eprintln!(
            "REFUSED_NO_JURISDICTION: no scan roots derivable (cargo metadata unavailable, no crates/*/Cargo.toml found). A green gate with empty jurisdiction is not evidence."
        );
        process::exit(2);
    }

    scan_dependencies(&mut findings);
    eprintln!(
        "NOTE: CHEAT-014 (reachable dependency branch, object-code layer) is not implemented by this scanner; dependency discovery only."
    );

    let mut per_root: Vec<(PathBuf, usize)> = roots.iter().map(|r| (r.clone(), 0)).collect();

    for (root, count) in per_root.iter_mut() {
        for entry in WalkDir::new(root)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "rs"))
        {
            let path = entry.path();
            *count += 1;
            total_files += 1;
            match fs::read_to_string(path) {
                Ok(src) => {
                    // 1. Text-based scans
                    scan_file_text_rules(&src, path, &mut findings);

                    // 2. AST-based scans
                    if let Ok(syntax) = syn::parse_file(&src) {
                        let mut visitor = SynCheatVisitor {
                            path,
                            findings: Vec::new(),
                            functions: Vec::new(),
                        };
                        visitor.visit_file(&syntax);

                        check_circular_oracles(&visitor.functions, path, &mut visitor.findings);
                        check_mutation_before_admission(
                            &visitor.functions,
                            path,
                            &mut visitor.findings,
                        );
                        findings.extend(visitor.findings);
                    }
                }
                Err(e) => {
                    eprintln!("ERROR: Failed to read {}: {}", path.display(), e);
                }
            }
        }
    }

    // Jurisdiction evidence accompanies every outcome (green or red): which
    // roots were inspected and how many files each contributed.
    for (root, count) in &per_root {
        eprintln!(
            "jurisdiction: {} — {} file(s) inspected",
            root.display(),
            count
        );
    }

    if total_files == 0 {
        eprintln!(
            "REFUSED_EMPTY_JURISDICTION: 0 .rs files inspected across {} root(s); refusing to emit a green gate.",
            per_root.len()
        );
        process::exit(2);
    }

    // AGENTS.md §17: no baseline suppression without a separately admitted
    // waiver artifact. `cheat-waivers.toml` in the repo root is that artifact;
    // waived findings print distinctly and never vanish, and a waiver that no
    // longer matches any finding is itself a finding (anti-vacuity: a probe
    // with no witnessed firing carries no bits).
    let waivers = load_waivers();
    let mut waived_out: Vec<(String, &Waiver)> = Vec::new();
    let mut blocking: Vec<String> = Vec::new();
    let mut matched: Vec<bool> = vec![false; waivers.len()];
    for f in findings {
        let hit = waivers.iter().enumerate().find(|(i, w)| {
            !matched[*i] && f.contains(&format!("CHEAT[{}]:", w.rule)) && f.contains(&w.path)
        });
        match hit {
            Some((i, w)) => {
                matched[i] = true;
                waived_out.push((f, w));
            }
            None => blocking.push(f),
        }
    }
    for (i, w) in waivers.iter().enumerate() {
        if !matched[i] {
            blocking.push(format!(
                "CHEAT[WAIVER-STALE]: {} — waiver for rule {} matches no current finding; remove the entry (a suppression with no witnessed firing carries no bits)",
                w.path, w.rule
            ));
        }
    }
    for (f, w) in &waived_out {
        eprintln!("WAIVED{} — waiver: {} ({})", f, w.admitted_by, w.reason);
    }

    if !blocking.is_empty() {
        for f in &blocking {
            eprintln!("{}", f);
        }
        eprintln!(
            "\n{} cheat finding(s). Fix before committing. ({} waived by admitted artifact)",
            blocking.len(),
            waived_out.len()
        );
        process::exit(1);
    }

    if waived_out.is_empty() {
        println!(
            "OK: no cheat patterns detected across {} algorithm files.",
            total_files
        );
    } else {
        println!(
            "OK: no unwaived cheat patterns across {} algorithm files ({} waived by admitted artifact).",
            total_files,
            waived_out.len()
        );
    }
}

/// One admitted suppression from `cheat-waivers.toml` (repo root, CWD).
struct Waiver {
    rule: String,
    path: String,
    reason: String,
    admitted_by: String,
}

/// Minimal line-based parser for the waiver artifact — no new dependencies.
/// Fail-closed: an unparsable or missing file yields zero waivers, and a
/// malformed non-empty file is reported on stderr.
fn load_waivers() -> Vec<Waiver> {
    let Ok(raw) = fs::read_to_string("cheat-waivers.toml") else {
        return Vec::new();
    };
    let mut waivers = Vec::new();
    let mut current: Option<PartialWaiver> = None;
    for (lineno, line) in raw.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line == "[[waiver]]" {
            if let Some(c) = current.take() {
                if let Some(w) = c.finish(lineno) {
                    waivers.push(w);
                }
            }
            current = Some(PartialWaiver::default());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            eprintln!(
                "WAIVER-WARN: cheat-waivers.toml:{} — not key = \"value\"",
                lineno + 1
            );
            continue;
        };
        let value = value.trim().trim_matches('"').to_string();
        if let Some(c) = current.as_mut() {
            match key.trim() {
                "rule" => c.rule = Some(value),
                "path" => c.path = Some(value),
                "reason" => c.reason = Some(value),
                "admitted_by" => c.admitted_by = Some(value),
                _ => {}
            }
        }
    }
    if let Some(c) = current.take() {
        if let Some(w) = c.finish(raw.lines().count()) {
            waivers.push(w);
        }
    }
    waivers
}

#[derive(Default)]
struct PartialWaiver {
    rule: Option<String>,
    path: Option<String>,
    reason: Option<String>,
    admitted_by: Option<String>,
}

impl PartialWaiver {
    fn finish(self, lineno: usize) -> Option<Waiver> {
        match (self.rule, self.path, self.reason, self.admitted_by) {
            (Some(rule), Some(path), Some(reason), Some(admitted_by)) => Some(Waiver {
                rule,
                path,
                reason,
                admitted_by,
            }),
            _ => {
                eprintln!(
                    "WAIVER-WARN: cheat-waivers.toml — incomplete [[waiver]] block near line {} (needs rule, path, reason, admitted_by); entry ignored",
                    lineno
                );
                None
            }
        }
    }
}

/// True when `sub` occurs within `max_gap` bytes after any occurrence of
/// `needle` — used to detect an exact-variant assertion surface
/// (`assert_eq!(x, Enum::Variant ..)`, `Err(Path::Variant)`) near a pattern.
fn window_contains(hay: &str, needle: &str, sub: &str, max_gap: usize) -> bool {
    let mut from = 0;
    while let Some(rel) = hay[from..].find(needle) {
        let start = from + rel + needle.len();
        let end = (start + max_gap).min(hay.len());
        if hay[start..end].contains(sub) {
            return true;
        }
        from = start;
    }
    false
}
