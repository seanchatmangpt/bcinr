use bcinr_core::logic::mask::select_u32;
use prettytable::{row, Table};
use std::time::Instant;

// Example function to benchmark
fn benchmark_primitive(name: &str, func: fn()) -> (String, String) {
    let start = Instant::now();
    func();
    let duration = start.elapsed();
    (name.to_string(), format!("{:?}", duration))
}

fn main() {
    let mut table = Table::new();
    table.add_row(row!["Primitive", "Latency"]);

    // Example entries
    let (name, lat) = benchmark_primitive("select_u32", || {
        let _ = select_u32(0xFF, 1, 2);
    });
    table.add_row(row![name, lat]);

    table.printstd();
}
