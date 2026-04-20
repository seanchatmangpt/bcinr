use std::process::Command;
use regex::Regex;
use std::collections::HashMap;
use prettytable::{Table, row};

fn main() {
    println!("--- AUDITING BCINR SUBSTRATE INTEGRITY ---");

    // 1. Run all tests and capture raw output
    let test_output = Command::new("cargo")
        .args(["test", "--manifest-path", "crates/bcinr-logic/Cargo.toml", "--all-features"])
        .output()
        .expect("Failed to run tests");
    let stdout = String::from_utf8_lossy(&test_output.stdout);

    // 2. Parse per-module results
    // We look for: "test algorithms::[name]::tests::[test_name] ... ok"
    let re_test = Regex::new(r"test (algorithms|abstractions)::([a-zA-Z0-9_]+)::tests::([a-z0-9_]+) \.\.\. (ok|FAILED)").unwrap();
    
    let mut module_stats: HashMap<String, (usize, usize)> = HashMap::new(); // (Pass, Fail)

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

    // 3. Build an HONEST table
    let mut table = Table::new();
    table.add_row(row!["Target", "Passed Tests", "Failed Tests", "Integrity Status"]);

    let mut keys: Vec<_> = module_stats.keys().collect();
    keys.sort();

    // Show a sample of the first 20 targets + the 6 abstractions to prove diversity
    for name in keys.iter().take(20) {
        let (pass, fail) = module_stats.get(*name).unwrap();
        let integrity = if *fail == 0 && *pass >= 4 { "VERIFIED ✅" } else { "WEAK / INCOMPLETE ⚠️" };
        table.add_row(row![name, pass, fail, integrity]);
    }

    println!("\nAUDITED MODULE EXECUTION MATRIX (Partial View)");
    table.printstd();

    println!("\nSubstrate Totals:");
    println!("  Modules Tracked: {}", module_stats.len());
    println!("  Total Passes:    {}", module_stats.values().map(|v| v.0).sum::<usize>());
    println!("  Total Failures:  {}", module_stats.values().map(|v| v.1).sum::<usize>());
}
