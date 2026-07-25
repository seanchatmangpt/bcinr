#![allow(clippy::cmp_owned)]
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RuleId {
    SelfCancelingOperations,
    CircularOracle,
    MagicConstants,
    ArtificialFileInflation,
    BoilerplateVerificationClaims,
    ScannerEvasion,
    DeadPathCompliance,
    BenchmarkTheater,
    MutantTheater,
    GateJurisdictionTheater,
    ReachableDependencyBranch,
    MutationBeforeAdmission,
    RejectionStateDrift,
    BlackBoxBranchlessnessClaim,
}

// Fixtures for AST/Text tests
const FIXTURE_CHEAT_001_A: &str = "
pub fn self_cancel_xor(x: u32) -> u32 {
    x ^ x
}
";

const FIXTURE_CHEAT_001_B: &str = "
pub fn self_cancel_add_xor(x: u32, y: u32) -> u32 {
    x.wrapping_add(y) ^ x
}
";

const FIXTURE_CHEAT_002: &str = "
pub fn my_algo(val: u64, aux: u64) -> u64 {
    val + aux
}
fn my_algo_reference(val: u64, aux: u64) -> u64 {
    val + aux
}
";

const FIXTURE_CHEAT_003: &str = "
pub fn uses_magic() -> u32 {
    let x = 3735928559; // 0xDEADBEEF
    x
}
";

const FIXTURE_CHEAT_004: &str = "
// -----------------------------------------------------------------------------
// PADDING ENSURING FILE LENGTH REQUIREMENT (>= 100 LINES)
// -----------------------------------------------------------------------------
";

const FIXTURE_CHEAT_005: &str = "
// Hoare-logic Verification Line 1: Branchless path is the unique solution to the state constraints of
// Hoare-logic Verification Line 2: Branchless path is the unique solution to the state constraints of
// Hoare-logic Verification Line 3: Branchless path is the unique solution to the state constraints of
// Hoare-logic Verification Line 4: Branchless path is the unique solution to the state constraints of
// Hoare-logic Verification Line 5: Branchless path is the unique solution to the state constraints of
";

const FIXTURE_CHEAT_006: &str = "
macro_rules! check_evasion {
    ($x:expr) => {
        if $x {
            42
        } else {
            0
        }
    };
}
";

const FIXTURE_CHEAT_007: &str = "
pub fn dummy_branchless() -> u32 { 1 }
pub fn run() {
    if false {
        dummy_branchless();
    }
}
";

const FIXTURE_CHEAT_008: &str = "
pub mod bench {
    use super::*;
    pub fn bench_my_algo(c: &mut Criterion) {
        c.bench_function(\"my_algo_branchless\", |b| {
            b.iter(|| {
                let res = allocate_branchless(42);
                res
            })
        });
    }
}
";

const FIXTURE_CHEAT_009: &str = "
#[cfg(test)]
mod tests {
    #[test]
    fn test_mutant() {
        assert_ne!(baseline, mutant);
    }
}
";

const FIXTURE_CHEAT_031: &str = "
/// Documenting that black_box guarantees branchlessness.
pub fn doc_claim() {}
";

// We mock the checking logic here to verify rules
fn run_mock_scanner(src: &str, rule: RuleId) -> bool {
    let normalized = src.to_lowercase();
    match rule {
        RuleId::SelfCancelingOperations => {
            if let Ok(syntax) = syn::parse_file(src) {
                struct CancelVisitor {
                    found: bool,
                }
                impl<'ast> syn::visit::Visit<'ast> for CancelVisitor {
                    fn visit_expr(&mut self, i: &'ast syn::Expr) {
                        if let syn::Expr::Binary(b) = i {
                            if matches!(b.op, syn::BinOp::BitXor(_) | syn::BinOp::Sub(_)) {
                                let left = &b.left;
                                let right = &b.right;
                                let left_str = quote::quote!(#left).to_string().replace(" ", "");
                                let right_str = quote::quote!(#right).to_string().replace(" ", "");
                                if left_str == right_str {
                                    self.found = true;
                                }
                                if let syn::Expr::MethodCall(mc) = &*b.left {
                                    if mc.method == "wrapping_add" {
                                        let receiver = &mc.receiver;
                                        let rec_str =
                                            quote::quote!(#receiver).to_string().replace(" ", "");
                                        if rec_str == right_str {
                                            self.found = true;
                                        }
                                    }
                                }
                            }
                        }
                        syn::visit::visit_expr(self, i);
                    }
                }
                let mut v = CancelVisitor { found: false };
                syn::visit::Visit::visit_file(&mut v, &syntax);
                return v.found;
            }
        }
        RuleId::CircularOracle => {
            if let Ok(syntax) = syn::parse_file(src) {
                struct OracleVisitor {
                    functions: std::collections::HashMap<String, String>,
                    circular: bool,
                }
                impl<'ast> syn::visit::Visit<'ast> for OracleVisitor {
                    fn visit_item_fn(&mut self, i: &'ast syn::ItemFn) {
                        let name = i.sig.ident.to_string();
                        let block = &i.block;
                        let body = quote::quote!(#block).to_string().replace(" ", "");
                        self.functions.insert(name, body);
                        syn::visit::visit_item_fn(self, i);
                    }
                }
                let mut v = OracleVisitor {
                    functions: std::collections::HashMap::new(),
                    circular: false,
                };
                syn::visit::Visit::visit_file(&mut v, &syntax);
                for (name, body) in &v.functions {
                    if name.ends_with("_reference") {
                        let base = name.trim_end_matches("_reference");
                        if let Some(p_body) = v.functions.get(base) {
                            if body == p_body {
                                v.circular = true;
                            }
                        }
                    }
                }
                return v.circular;
            }
        }
        RuleId::MagicConstants => {
            if let Ok(syntax) = syn::parse_file(src) {
                struct MagicVisitor {
                    found: bool,
                }
                impl<'ast> syn::visit::Visit<'ast> for MagicVisitor {
                    fn visit_expr(&mut self, i: &'ast syn::Expr) {
                        if let syn::Expr::Lit(l) = i {
                            if let syn::Lit::Int(li) = &l.lit {
                                if let Ok(val) = li.base10_parse::<u64>() {
                                    if val == 3735928559 || val == 3405691582 {
                                        self.found = true;
                                    }
                                }
                            }
                        }
                        syn::visit::visit_expr(self, i);
                    }
                }
                let mut v = MagicVisitor { found: false };
                syn::visit::Visit::visit_file(&mut v, &syntax);
                return v.found;
            }
        }
        RuleId::ArtificialFileInflation => {
            return src.contains("PADDING ENSURING FILE LENGTH REQUIREMENT");
        }
        RuleId::BoilerplateVerificationClaims => {
            let mut hoare_count = 0;
            for line in src.lines() {
                if line.contains("Hoare-logic Verification Line")
                    && line.contains(
                        "Branchless path is the unique solution to the state constraints of",
                    )
                {
                    hoare_count += 1;
                }
            }
            return hoare_count >= 5;
        }
        RuleId::ScannerEvasion => {
            if let Ok(syntax) = syn::parse_file(src) {
                struct EvasionVisitor {
                    found: bool,
                }
                impl<'ast> syn::visit::Visit<'ast> for EvasionVisitor {
                    fn visit_item_macro(&mut self, i: &'ast syn::ItemMacro) {
                        if let Some(ident) = i.mac.path.get_ident() {
                            if ident.to_string() == "macro_rules" {
                                let mac_str = quote::quote!(#i).to_string();
                                if mac_str.contains("if") || mac_str.contains("match") {
                                    self.found = true;
                                }
                            }
                        }
                    }
                }
                let mut v = EvasionVisitor { found: false };
                syn::visit::Visit::visit_file(&mut v, &syntax);
                return v.found;
            }
        }
        RuleId::DeadPathCompliance => {
            return src.contains("if false {") && src.contains("dummy_branchless");
        }
        RuleId::BenchmarkTheater => {
            if let Ok(syntax) = syn::parse_file(src) {
                struct BenchVisitor {
                    found: bool,
                }
                impl<'ast> syn::visit::Visit<'ast> for BenchVisitor {
                    fn visit_expr(&mut self, i: &'ast syn::Expr) {
                        if let syn::Expr::MethodCall(mc) = i {
                            if mc.method == "bench_function" || mc.method == "iter" {
                                let arg_str = quote::quote!(#mc).to_string();
                                if arg_str.contains("branchless") && !arg_str.contains("black_box")
                                {
                                    self.found = true;
                                }
                            }
                        }
                        syn::visit::visit_expr(self, i);
                    }
                }
                let mut v = BenchVisitor { found: false };
                syn::visit::Visit::visit_file(&mut v, &syntax);
                return v.found;
            }
        }
        RuleId::MutantTheater => {
            return src.contains("assert_ne!") && !src.contains("StabilityRefusal");
        }
        RuleId::BlackBoxBranchlessnessClaim => {
            return normalized.contains("black_box guarantees");
        }
        _ => {}
    }
    false
}

#[test]
fn test_cheat_rules_matrix() {
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_001_A,
        RuleId::SelfCancelingOperations
    ));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_001_B,
        RuleId::SelfCancelingOperations
    ));
    assert!(run_mock_scanner(FIXTURE_CHEAT_002, RuleId::CircularOracle));
    assert!(run_mock_scanner(FIXTURE_CHEAT_003, RuleId::MagicConstants));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_004,
        RuleId::ArtificialFileInflation
    ));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_005,
        RuleId::BoilerplateVerificationClaims
    ));
    assert!(run_mock_scanner(FIXTURE_CHEAT_006, RuleId::ScannerEvasion));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_007,
        RuleId::DeadPathCompliance
    ));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_008,
        RuleId::BenchmarkTheater
    ));
    assert!(run_mock_scanner(FIXTURE_CHEAT_009, RuleId::MutantTheater));
    assert!(run_mock_scanner(
        FIXTURE_CHEAT_031,
        RuleId::BlackBoxBranchlessnessClaim
    ));
}
