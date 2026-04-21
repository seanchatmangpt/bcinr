use std::collections::HashSet;
use std::fs;
use syn::visit::Visit;
use syn::{ItemFn, Visibility};
use walkdir::WalkDir;

#[derive(Default)]
struct FnVisitor {
    pub_fns: HashSet<String>,
}

impl<'ast> Visit<'ast> for FnVisitor {
    fn visit_item_fn(&mut self, i: &'ast ItemFn) {
        if let Visibility::Public(_) = i.vis {
            let name = i.sig.ident.to_string();
            if !name.ends_with("_gate") 
                && !["new", "new_checked", "default", "len", "is_empty", "in_bounds", "check_integrity", "check_substrate_integrity", "vision_integrity_check"].contains(&name.as_str()) 
                && !name.starts_with("bench_") {
                self.pub_fns.insert(name);
            }
        }
        syn::visit::visit_item_fn(self, i);
    }
}

fn main() {
    let mut logic_fns = HashSet::new();

    for entry in WalkDir::new("crates/bcinr-logic/src") {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = fs::read_to_string(entry.path()).unwrap();
            // Don't parse test files
            if content.contains("#[cfg(test)]") {
                // remove test modules for parsing (crude but works)
                let cleaned = content.split("#[cfg(test)]").next().unwrap_or("").to_string();
                if let Ok(file) = syn::parse_file(&cleaned) {
                    let mut visitor = FnVisitor::default();
                    visitor.visit_file(&file);
                    logic_fns.extend(visitor.pub_fns);
                }
            } else {
                if let Ok(file) = syn::parse_file(&content) {
                    let mut visitor = FnVisitor::default();
                    visitor.visit_file(&file);
                    logic_fns.extend(visitor.pub_fns);
                }
            }
        }
    }

    let mut bench_content = String::new();
    for entry in WalkDir::new("bcinr-bench/benches") {
        let entry = entry.unwrap();
        if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
            bench_content.push_str(&fs::read_to_string(entry.path()).unwrap());
            bench_content.push('\n');
        }
    }

    let mut missing = Vec::new();
    for fn_name in &logic_fns {
        let signature_call = format!("{}(", fn_name);
        let bench_call = format!("\"{}\"", fn_name);
        if !bench_content.contains(&signature_call) && !bench_content.contains(&bench_call) {
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
