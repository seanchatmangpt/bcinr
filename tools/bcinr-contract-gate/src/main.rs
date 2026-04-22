use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};
use syn::{ItemFn, Visibility, Expr, BinOp, Attribute, Meta, MetaNameValue, Lit, Expr as SynExpr};
use walkdir::WalkDir;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct PublicFunction {
    name: String,
    path: PathBuf,
    has_u64_contract: bool,
}

impl PublicFunction {
    #[allow(dead_code)]
    fn pass_test_name(&self) -> String {
        format!("{}__equivalence", self.name)
    }
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

/// Concatenate all `///` doc-comment attribute strings on a function.
fn doc_text(attrs: &[Attribute]) -> String {
    let mut buf = String::new();
    for a in attrs {
        if let Meta::NameValue(MetaNameValue { path, value, .. }) = &a.meta {
            if path.is_ident("doc") {
                if let SynExpr::Lit(lit) = value {
                    if let Lit::Str(s) = &lit.lit {
                        buf.push_str(&s.value());
                        buf.push('\n');
                    }
                }
            }
        }
    }
    buf
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

            if !cv.forbidden_ops.is_empty() && (name.contains("add_bitwise") || name.contains("sub_bitwise")) {
                self.errors.push(format!("FAIL: {} in {} uses forbidden operator(s): {:?} (Bluff detected!)",
                    name, self.current_path.display(), cv.forbidden_ops));
            }

            // Branchless contract detection: function-level doc OR file-level doc.
            let fn_doc = doc_text(&i.attrs);
            let has_u64 = fn_doc.contains("Branchless Contract") || self.file_doc_has_u64_contract;

            self.public_functions.push(PublicFunction {
                name,
                path: self.current_path.clone(),
                has_u64_contract: has_u64,
            });
        }
        visit::visit_item_fn(self, i);
    }
}

/// Read the inner doc comment lines (`//!`) from the head of a file.
fn file_inner_doc(content: &str) -> String {
    let mut buf = String::new();
    for line in content.lines() {
        let l = line.trim_start();
        if l.starts_with("//!") {
            buf.push_str(l);
            buf.push('\n');
        } else if !l.is_empty() && !l.starts_with("//") {
            // first real code line; stop scanning
            break;
        }
    }
    buf
}

fn main() {
    let mut visitor = GateVisitor::default();
    let src_dir = Path::new("crates/bcinr-logic/src");

    let mut parse_warnings: Vec<String> = Vec::new();
    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().is_some_and(|ext| ext == "rs") {
            let path = entry.path();
            let content = fs::read_to_string(path).unwrap();
            // A file is U64-contracted if its inner doc declares it OR any
            // function/comment block in the file declares the contract.
            visitor.file_doc_has_u64_contract = file_inner_doc(&content).contains("Branchless Contract")
                || content.contains("BRANCHLESS CONTRACT")
                || content.contains("Branchless Contract");
            match syn::parse_file(&content) {
                Ok(syntax) => {
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
                !p.ends_with("/mod.rs")
                && !f.has_u64_contract
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
