/// Quick timing test for sigmoid_sat_u32 and exp2_u64_fixed
/// Compile: rustc -O quick_time_test.rs --extern bcinr_logic=target/release/libbcinr_logic.rlib
/// Run: ./quick_time_test

extern crate bcinr_logic;

use std::time::Instant;
use std::hint::black_box;

fn main() {
    println!("Timing approximation functions");
    println!("==============================\n");

    // Time sigmoid_sat_u32
    println!("sigmoid_sat_u32:");
    time_function("avg", 42u64, 1337u64, bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32);
    time_function("min", 0u64, 0u64, bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32);
    time_function("max", 0xFFFFFFFF_u64, 0xFFFFFFFF_u64, bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32);

    // Time exp2_u64_fixed
    println!("\nexp2_u64_fixed:");
    time_function("avg", 42u64, 1337u64, bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed);
    time_function("min", 0u64, 0u64, bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed);
    time_function("max", u64::MAX, 0u64, bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed);
}

fn time_function<F>(label: &str, val: u64, aux: u64, f: F)
where
    F: Fn(u64, u64) -> u64,
{
    const ITERATIONS: u64 = 10_000_000;

    let start = Instant::now();
    let mut result = 0u64;

    for _ in 0..ITERATIONS {
        result = f(black_box(val), black_box(aux));
    }

    let elapsed = start.elapsed();
    let ns_per_iter = (elapsed.as_nanos() as f64) / (ITERATIONS as f64);

    println!("  {}_{}: {:.3} ns/iter (result: {})",
        std::any::type_name::<F>().split("::").last().unwrap_or("func"),
        label,
        ns_per_iter,
        result);
}
