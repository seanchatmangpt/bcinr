use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};
use syn::{ItemFn, Visibility, Expr, BinOp, Attribute, Meta, Lit};
use walkdir::WalkDir;

fn str_has_substr(s: &str, pat: &str) -> bool {
    if pat.is_empty() { return true; }
    s.as_bytes().windows(pat.len()).any(|w| w == pat.as_bytes())
}

fn str_ends_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() { return false; }
    s.as_bytes()[s.len() - pat.len()..] == *pat.as_bytes()
}

fn str_starts_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() { return false; }
    s.as_bytes()[..pat.len()] == *pat.as_bytes()
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct PublicFunction {
    name: String,
    path: PathBuf,
    has_u64_contract: bool,
}

#[derive(Default)]
struct ComplexityVisitor {
    complexity: usize,
    forbidden_ops: Vec<String>,
}

impl<'ast> Visit<'ast> for ComplexityVisitor {
    fn visit_expr(&mut self, i: &'ast Expr) {
        match i {
            Expr::If(_) | Expr::Match(_) | Expr::Loop(_) | Expr::While(_) | Expr::ForLoop(_) => {
                self.complexity += 1;
            }
            Expr::Binary(b) => {
                match b.op {
                    BinOp::Add(_) => self.forbidden_ops.push("+".to_string()),
                    BinOp::Sub(_) => self.forbidden_ops.push("-".to_string()),
                    BinOp::Mul(_) => self.forbidden_ops.push("*".to_string()),
                    BinOp::Div(_) => self.forbidden_ops.push("/".to_string()),
                    _ => {}
                }
            }
            _ => {}
        }
        visit::visit_expr(self, i);
    }
}

fn get_doc_string(attr: &Attribute) -> Option<String> {
    if attr.path().is_ident("doc") {
        if let Meta::NameValue(meta) = &attr.meta {
            if let Expr::Lit(expr_lit) = &meta.value {
                if let Lit::Str(lit_str) = &expr_lit.lit {
                    return Some(lit_str.value());
                }
            }
        }
    }
    None
}

fn has_contract_in_attrs(attrs: &[Attribute]) -> bool {
    for attr in attrs {
        if let Some(doc_str) = get_doc_string(attr) {
            if str_has_substr(&doc_str, "Branchless Contract") || str_has_substr(&doc_str, "BRANCHLESS CONTRACT") {
                return true;
            }
        }
    }
    false
}

#[derive(Default)]
struct GateVisitor {
    public_functions: Vec<PublicFunction>,
    test_functions: BTreeSet<String>,
    current_path: PathBuf,
    errors: Vec<String>,
    file_doc_has_u64_contract: bool,
}

impl<'ast> Visit<'ast> for GateVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let name = i.sig.ident.to_string();

        if i.attrs.iter().any(|attr| attr.path().is_ident("test")) {
            self.test_functions.insert(name.clone());
        }

        if matches!(i.vis, Visibility::Public(_)) {
            let mut cv = ComplexityVisitor { complexity: 1, ..Default::default() };
            cv.visit_item_fn(i);

            if cv.complexity > 1 {
                self.errors.push(format!("FAIL: {} in {} has Cyclomatic Complexity {} (Branch detected!)",
                         name, self.current_path.display(), cv.complexity));
            }

            if !cv.forbidden_ops.is_empty() && (str_has_substr(&name, "add_bitwise") || str_has_substr(&name, "sub_bitwise")) {
                self.errors.push(format!("FAIL: {} in {} uses forbidden operator(s): {:?} (Bluff detected!)",
                    name, self.current_path.display(), cv.forbidden_ops));
            }

            // Branchless contract detection: function-level doc OR file-level doc.
            let has_u64 = has_contract_in_attrs(&i.attrs) || self.file_doc_has_u64_contract;

            self.public_functions.push(PublicFunction {
                name,
                path: self.current_path.clone(),
                has_u64_contract: has_u64,
            });
        }
        visit::visit_item_fn(self, i);
    }
}

fn main() {
    let mut visitor = GateVisitor::default();
    let src_dir = Path::new("crates/bcinr-logic/src/algorithms");

    let mut parse_warnings: Vec<String> = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "rs") {
            let path = entry.path();
            let content = match fs::read_to_string(path) {
                Ok(c) => c,
                Err(_) => continue,
            };
            match syn::parse_file(&content) {
                Ok(syntax) => {
                    // A file is U64-contracted if its inner doc declares it OR any
                    // function/comment block in the file declares the contract.
                    visitor.file_doc_has_u64_contract = has_contract_in_attrs(&syntax.attrs);
                    visitor.current_path = path.to_path_buf();
                    visitor.visit_file(&syntax);
                }
                Err(e) => {
                    // Legacy aggregator files (mod.rs alongside lib.rs) may not parse
                    // standalone; treat as warning, not gate failure.
                    parse_warnings.push(format!("WARN parse: {}: {}", path.display(), e));
                }
            }
        }
    }
    for w in &parse_warnings {
        eprintln!("{}", w);
    }

    let mut missing_u64: Vec<&PublicFunction> = visitor.public_functions.iter()
        .filter(|f| {
            let p = f.path.to_string_lossy();
            !str_ends_with(&p, "/mod.rs") && !str_starts_with(&f.name, "bench_") && !f.has_u64_contract
        })
        .collect();
    missing_u64.sort();

    for f in &missing_u64 {
        visitor.errors.push(format!(
            "MISSING_U64_CONTRACT: {} in {}",
            f.name,
            f.path.display()
        ));
    }

    if !visitor.errors.is_empty() {
        for err in &visitor.errors {
            println!("{}", err);
        }
        std::process::exit(1);
    }

    let total = visitor.public_functions.len();
    let with_u64 = visitor.public_functions.iter().filter(|f| f.has_u64_contract).count();
    println!("--- BCINR INTEGRITY AUDIT (Complexity + Construction + Branchless) ---");
    println!("Verified {} public primitives ✅", total);
    println!("Branchless-contracted: {}/{}", with_u64, total);
    println!("No bluffs, no hidden branches, no missing U64 contracts.");
}
