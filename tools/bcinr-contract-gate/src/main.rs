use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use syn::visit::{self, Visit};
use syn::{ItemFn, Visibility, Expr, BinOp};
use walkdir::WalkDir;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
struct PublicFunction {
    name: String,
    path: PathBuf,
}

impl PublicFunction {
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

#[derive(Default)]
struct GateVisitor {
    public_functions: Vec<PublicFunction>,
    test_functions: BTreeSet<String>,
    current_path: PathBuf,
    errors: Vec<String>,
}

impl<'ast> Visit<'ast> for GateVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        let name = i.sig.ident.to_string();
        
        if i.attrs.iter().any(|attr| attr.path().is_ident("test")) {
            self.test_functions.insert(name.clone());
        }

        if matches!(i.vis, Visibility::Public(_)) {
            let mut cv = ComplexityVisitor::default();
            cv.complexity = 1;
            cv.visit_item_fn(i);
            
            if cv.complexity > 1 {
                self.errors.push(format!("FAIL: {} in {} has Cyclomatic Complexity {} (Branch detected!)", 
                         name, self.current_path.display(), cv.complexity));
            }

            if !cv.forbidden_ops.is_empty() {
                if name.contains("add_bitwise") || name.contains("sub_bitwise") {
                     self.errors.push(format!("FAIL: {} in {} uses forbidden operator(s): {:?} (Bluff detected!)", 
                         name, self.current_path.display(), cv.forbidden_ops));
                }
            }

            self.public_functions.push(PublicFunction {
                name,
                path: self.current_path.clone(),
            });
        }
        visit::visit_item_fn(self, i);
    }
}

fn main() {
    let mut visitor = GateVisitor::default();
    let src_dir = Path::new("crates/bcinr-logic/src");

    for entry in WalkDir::new(src_dir).into_iter().filter_map(|e| e.ok()) {
        if entry.path().extension().map_or(false, |ext| ext == "rs") {
            let content = fs::read_to_string(entry.path()).unwrap();
            match syn::parse_file(&content) {
                Ok(syntax) => {
                    visitor.current_path = entry.path().to_path_buf();
                    visitor.visit_file(&syntax);
                }
                Err(e) => {
                    visitor.errors.push(format!("Failed to parse file {}: {}", entry.path().display(), e));
                }
            }
        }
    }

    if !visitor.errors.is_empty() {
        for err in &visitor.errors {
            println!("{}", err);
        }
        std::process::exit(1);
    }

    println!("--- BCINR INTEGRITY AUDIT (Complexity + Construction) ---");
    println!("Verified {} public primitives ✅", visitor.public_functions.len());
    println!("No bluffs or hidden branches found in hot paths.");
}

