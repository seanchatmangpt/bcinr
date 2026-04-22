use prettytable::{row, Table};
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

fn module_has_u64_contract(module: &str) -> bool {
    let candidates = [
        format!("crates/bcinr-logic/src/algorithms/{}.rs", module),
        format!("crates/bcinr-logic/src/abstractions/{}.rs", module),
    ];
    for c in &candidates {
        if Path::new(c).exists() {
            if let Ok(_s) = fs::read_to_string(c) {
            }
        }
    }
    false
}

fn main() {
    println!("--- AUDITING BCINR SUBSTRATE INTEGRITY ---");

    let test_output = Command::new("cargo")
        .args(["test", "--manifest-path", "crates/bcinr-logic/Cargo.toml", "--all-features"])
        .output()
        .expect("Failed to run tests");
    let stdout = String::from_utf8_lossy(&test_output.stdout);

    let re_test = Regex::new(
        r"test (algorithms|abstractions|patterns)::([a-zA-Z0-9_]+)(?:::[a-zA-Z0-9_]+)?::tests::([a-z0-9_]+) \.\.\. (ok|FAILED)",
    )
    .unwrap();

    let mut module_stats: HashMap<String, (usize, usize)> = HashMap::new();

    for cap in re_test.captures_iter(&stdout) {
        let module_name = cap[2].to_string();
        let status = &cap[4];
        let stats = module_stats.entry(module_name).or_insert((0, 0));
        if status == "ok" {
            stats.0 += 1;
        } else {
            stats.1 += 1;
        }
    }

    let mut table = Table::new();
    table.add_row(row!["Target", "Passed", "Failed", "Integrity", "U64 Contract"]);

    let mut keys: Vec<_> = module_stats.keys().collect();
    keys.sort();

    let mut u64_total = 0usize;
    for name in keys.iter().take(30) {
        let (pass, fail) = module_stats.get(*name).unwrap();
        let integrity = if *fail == 0 && *pass >= 4 { "VERIFIED ✅" } else { "WEAK ⚠️" };
        let has_u64 = module_has_u64_contract(name);
        if has_u64 {
            u64_total += 1;
        }
        let u64_cell = if has_u64 { "YES ✅" } else { "NO ❌" };
        table.add_row(row![name, pass, fail, integrity, u64_cell]);
    }

    println!("\nAUDITED MODULE EXECUTION MATRIX (Top 30)");
    table.printstd();

    let total_modules = module_stats.len();
    let mut u64_global = 0usize;
    for name in module_stats.keys() {
        if module_has_u64_contract(name) {
            u64_global += 1;
        }
    }

    println!("\nSubstrate Totals:");
    println!("  Modules Tracked:        {}", total_modules);
    println!("  Total Passes:           {}", module_stats.values().map(|v| v.0).sum::<usize>());
    println!("  Total Failures:         {}", module_stats.values().map(|v| v.1).sum::<usize>());
    println!("  U64 Contract (sample):  {}/{}", u64_total, keys.iter().take(30).count());
    println!("  U64 Contract (global):  {}/{}", u64_global, total_modules);
}
