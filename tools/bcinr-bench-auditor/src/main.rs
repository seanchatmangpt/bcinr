use std::collections::HashSet;
use std::fs;
use syn::visit::Visit;
use syn::{ItemFn, Visibility, ItemMod, Ident, LitStr, Attribute};
use walkdir::WalkDir;

fn str_starts_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() { return false; }
    s.as_bytes()[..pat.len()] == *pat.as_bytes()
}

fn str_ends_with(s: &str, pat: &str) -> bool {
    if s.len() < pat.len() { return false; }
    s.as_bytes()[s.len() - pat.len()..] == *pat.as_bytes()
}

#[allow(clippy::unnecessary_get_then_check)]
fn is_cfg_test(attr: &Attribute) -> bool {
    if attr.path().is_ident("cfg") {
        let mut is_test = false;
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("test") {
                is_test = true;
            }
            Ok(())
        });
        is_test
    } else {
        false
    }
}

fn has_cfg_test_attr(attrs: &[Attribute]) -> bool {
    attrs.iter().any(is_cfg_test)
}

#[derive(Default)]
struct FnVisitor {
    pub_fns: HashSet<String>,
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if let Visibility::Public(_) = i.vis {
            let name = i.sig.ident.to_string();
            let ignored_names = ["new", "new_checked", "default", "len", "is_empty", "in_bounds", "check_integrity", "check_substrate_integrity", "vision_integrity_check"];
            let mut is_ignored = false;
            for ignored in &ignored_names {
                if name == *ignored {
                    is_ignored = true;
                }
            }
            if !str_ends_with(&name, "_gate") 
                && !is_ignored
                && !str_starts_with(&name, "bench_") {
                self.pub_fns.insert(name);
            }
        }
        syn::visit::visit_item_fn(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if !has_cfg_test_attr(&i.attrs) {
            syn::visit::visit_item_mod(self, i);
        }
    }
}

#[derive(Default)]
struct BenchVisitor {
    idents: HashSet<String>,
}

impl<'ast> Visit<'ast> for BenchVisitor {
    fn visit_ident(&mut self, i: &'ast Ident) {
        self.idents.insert(i.to_string());
        syn::visit::visit_ident(self, i);
    }

    fn visit_lit_str(&mut self, i: &'ast LitStr) {
        self.idents.insert(i.value());
        syn::visit::visit_lit_str(self, i);
    }

    fn visit_item_mod(&mut self, i: &'ast ItemMod) {
        if !has_cfg_test_attr(&i.attrs) {
            syn::visit::visit_item_mod(self, i);
        }
    }
}

#[allow(clippy::unnecessary_get_then_check)]
fn main() {
    let mut logic_fns = HashSet::new();

    // Limit bcinr-bench-auditor check directory to only crates/bcinr-logic/src/algorithms
    for entry in WalkDir::new("crates/bcinr-logic/src/algorithms") {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Ok(file) = syn::parse_file(&content) {
                let mut visitor = FnVisitor::default();
                visitor.visit_file(&file);
                logic_fns.extend(visitor.pub_fns);
            }
        }
    }

    let mut bench_idents = HashSet::new();
    for entry in WalkDir::new("bcinr-bench/benches") {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = match fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };
            if let Ok(file) = syn::parse_file(&content) {
                let mut visitor = BenchVisitor::default();
                visitor.visit_file(&file);
                bench_idents.extend(visitor.idents);
            }
        }
    }

    let mut missing = Vec::new();
    for fn_name in &logic_fns {
        if bench_idents.get(fn_name).is_none() {
            missing.push(fn_name.clone());
        }
    }

    missing.sort();

    if missing.is_empty() {
        println!("SUCCESS: All {} public capabilities are benchmarked!", logic_fns.len());
    } else {
        println!("FAILED: Found {} public functions NOT benchmarked out of {}:", missing.len(), logic_fns.len());
        for m in &missing {
            println!("  - {}", m);
        }
        std::process::exit(1);
    }
}
