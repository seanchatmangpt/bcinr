use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};
use syn::{Attribute, Expr, ImplItemFn, ItemFn, Meta, Visibility};
use walkdir::WalkDir;

const AUTHORITATIVE_ROOTS: &[&str] = &["allocate", "evaluate_calibration"];

fn str_has_substr(s: &str, pat: &str) -> bool {
    if pat.is_empty() {
        return true;
    }
    s.as_bytes().windows(pat.len()).any(|w| w == pat.as_bytes())
}

fn str_ends_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() {
        return false;
    }
    s.as_bytes()[s.len() - pat.len()..] == *pat.as_bytes()
}

fn str_starts_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() {
        return false;
    }
    s.as_bytes()[..pat.len()] == *pat.as_bytes()
}

fn get_doc_string(attr: &Attribute) -> Option<String> {
    if attr.path().is_ident("doc") {
        if let Meta::NameValue(meta) = &attr.meta {
            if let Expr::Lit(expr_lit) = &meta.value {
                if let syn::Lit::Str(lit_str) = &expr_lit.lit {
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
            if str_has_substr(&doc_str, "Branchless Contract")
                || str_has_substr(&doc_str, "BRANCHLESS CONTRACT")
                || str_has_substr(&doc_str, "u64_contract!")
            {
                return true;
            }
        }
    }
    false
}

#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    path: PathBuf,
    attrs: Vec<Attribute>,
    vis: Visibility,
    complexity: usize,
    has_contract: bool,
    callees: Vec<String>,
    forbidden_ops: Vec<String>,
}

struct CalleeVisitor {
    callees: Vec<String>,
    complexity: usize,
    forbidden_ops: Vec<String>,
}

impl<'ast> Visit<'ast> for CalleeVisitor {
    fn visit_expr(&mut self, i: &'ast Expr) {
        match i {
            Expr::If(_) | Expr::Match(_) | Expr::Loop(_) | Expr::While(_) | Expr::ForLoop(_) => {
                self.complexity += 1;
            }
            Expr::Try(_) => {
                self.complexity += 1;
            }
            Expr::MethodCall(mc) => {
                let m = mc.method.to_string();
                if m == "unwrap" || m == "expect" || m == "unwrap_or" || m == "unwrap_or_else" {
                    self.complexity += 1;
                }
                self.callees.push(m);
            }
            Expr::Call(c) => {
                if let Expr::Path(ep) = &*c.func {
                    if let Some(ident) = ep.path.get_ident() {
                        self.callees.push(ident.to_string());
                    }
                }
            }
            Expr::Binary(b) => match b.op {
                syn::BinOp::Add(_) => self.forbidden_ops.push("+".to_string()),
                syn::BinOp::Sub(_) => self.forbidden_ops.push("-".to_string()),
                syn::BinOp::Mul(_) => self.forbidden_ops.push("*".to_string()),
                syn::BinOp::Div(_) => self.forbidden_ops.push("/".to_string()),
                _ => {}
            },
            _ => {}
        }
        visit::visit_expr(self, i);
    }
}

struct CallGraphVisitor {
    current_path: PathBuf,
    functions: Vec<FunctionInfo>,
    file_doc_has_contract: bool,
}

impl<'ast> Visit<'ast> for CallGraphVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let name = i.sig.ident.to_string();
        let mut cv = CalleeVisitor {
            callees: Vec::new(),
            complexity: 1,
            forbidden_ops: Vec::new(),
        };
        cv.visit_item_fn(i);

        let has_contract = has_contract_in_attrs(&i.attrs) || self.file_doc_has_contract;

        self.functions.push(FunctionInfo {
            name,
            path: self.current_path.clone(),
            attrs: i.attrs.clone(),
            vis: i.vis.clone(),
            complexity: cv.complexity,
            has_contract,
            callees: cv.callees,
            forbidden_ops: cv.forbidden_ops,
        });

        visit::visit_item_fn(self, i);
    }

    fn visit_impl_item_fn(&mut self, i: &'ast ImplItemFn) {
        let name = i.sig.ident.to_string();
        let mut cv = CalleeVisitor {
            callees: Vec::new(),
            complexity: 1,
            forbidden_ops: Vec::new(),
        };
        cv.visit_impl_item_fn(i);

        let has_contract = has_contract_in_attrs(&i.attrs) || self.file_doc_has_contract;

        self.functions.push(FunctionInfo {
            name,
            path: self.current_path.clone(),
            attrs: i.attrs.clone(),
            vis: i.vis.clone(),
            complexity: cv.complexity,
            has_contract,
            callees: cv.callees,
            forbidden_ops: cv.forbidden_ops,
        });

        visit::visit_impl_item_fn(self, i);
    }
}

fn should_ignore(attrs: &[Attribute], name: &str, path: &Path) -> bool {
    let p = path.to_string_lossy();
    if p.contains("/tests/") || p.contains("/benches/") {
        return true;
    }
    if name.ends_with("_reference") || name.ends_with("_test") {
        return true;
    }
    for attr in attrs {
        let attr_str = quote::quote!(#attr).to_string();
        if attr_str.contains("test") || attr_str.contains("bench") {
            return true;
        }
    }
    false
}

fn is_checked(f: &FunctionInfo, reachable: &BTreeSet<String>, arg_specified: bool) -> bool {
    if arg_specified {
        return true;
    }
    let file_name = f.path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if f.name.starts_with("temp_") || file_name.contains("temp_") {
        return true;
    }
    reachable.contains(&f.name)
}

fn main() {
    let arg = std::env::args().nth(1);
    // If no path is specified, scan both crates/bcinr-logic and crates/bcinr-cmca to build the full call graph!
    let scan_dirs = match &arg {
        Some(p) => vec![PathBuf::from(p)],
        None => vec![
            PathBuf::from("crates/bcinr-logic"),
            PathBuf::from("crates/bcinr-cmca"),
        ],
    };

    let mut visitor = CallGraphVisitor {
        current_path: PathBuf::new(),
        functions: Vec::new(),
        file_doc_has_contract: false,
    };

    for src_dir in &scan_dirs {
        for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.path().extension().is_some_and(|ext| ext == "rs") {
                let path = entry.path();
                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                if let Ok(syntax) = syn::parse_file(&content) {
                    visitor.file_doc_has_contract = has_contract_in_attrs(&syntax.attrs);
                    visitor.current_path = path.to_path_buf();
                    visitor.visit_file(&syntax);
                }
            }
        }
    }

    // Build the reachability graph from AUTHORITATIVE_ROOTS
    let mut func_map: BTreeMap<String, &FunctionInfo> = BTreeMap::new();
    for f in &visitor.functions {
        // If there are duplicate names, keep the one that is NOT ignored
        if !should_ignore(&f.attrs, &f.name, &f.path) {
            func_map.insert(f.name.clone(), f);
        }
    }

    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();

    for root in AUTHORITATIVE_ROOTS {
        if func_map.contains_key(*root) {
            queue.push_back((*root).to_string());
            reachable.insert((*root).to_string());
        }
    }

    while let Some(node) = queue.pop_front() {
        if let Some(f_info) = func_map.get(&node) {
            for callee in &f_info.callees {
                if !reachable.contains(callee) && func_map.contains_key(callee) {
                    reachable.insert(callee.clone());
                    queue.push_back(callee.clone());
                }
            }
        }
    }

    let mut errors = Vec::new();
    let mut public_functions = Vec::new();

    for f in &visitor.functions {
        // Skip if not checked based on reachability / temp name
        if !is_checked(f, &reachable, arg.is_some()) {
            continue;
        }

        if should_ignore(&f.attrs, &f.name, &f.path) {
            continue;
        }

        if f.complexity > 1 {
            errors.push(format!(
                "FAIL: {} in {} has Cyclomatic Complexity {} (Branch detected!)",
                f.name,
                f.path.display(),
                f.complexity
            ));
        }

        if !f.forbidden_ops.is_empty() {
            if f.name.contains("add_bitwise") || f.name.contains("sub_bitwise") {
                errors.push(format!(
                    "FAIL: {} in {} uses forbidden operator(s): {:?} (Bluff detected!)",
                    f.name,
                    f.path.display(),
                    f.forbidden_ops
                ));
            }
        }

        if matches!(f.vis, Visibility::Public(_)) {
            public_functions.push(f);
        }
    }

    let mut missing_u64 = Vec::new();
    for f in &public_functions {
        let p = f.path.to_string_lossy();
        if !str_ends_with(&p, "/mod.rs") && !str_starts_with(&f.name, "bench_") && !f.has_contract {
            missing_u64.push(f);
        }
    }
    missing_u64.sort_by_key(|f| &f.name);

    for f in &missing_u64 {
        errors.push(format!(
            "MISSING_U64_CONTRACT: {} in {}",
            f.name,
            f.path.display()
        ));
    }

    if !errors.is_empty() {
        for err in &errors {
            println!("{}", err);
        }
        std::process::exit(1);
    }

    println!("--- BCINR INTEGRITY AUDIT (Complexity + Construction + Branchless) ---");
    println!(
        "Verified {} reachable public primitives ✅",
        public_functions.len()
    );
    println!(
        "Branchless-contracted: {}/{}",
        public_functions.iter().filter(|f| f.has_contract).count(),
        public_functions.len()
    );
    println!("No bluffs, no hidden branches, no missing U64 contracts.");
}
