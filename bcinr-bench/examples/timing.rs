use bcinr_logic::algorithms::sigmoid_sat_u32::sigmoid_sat_u32;
use bcinr_logic::algorithms::exp2_u64_fixed::exp2_u64_fixed;
use std::time::Instant;
use std::hint::black_box;

fn main() {
    const ITERATIONS: u64 = 10_000_000;

    println!("Approximation Function Benchmarks");
    println!("=================================\n");

    // sigmoid_sat_u32 tests
    println!("sigmoid_sat_u32:");
    benchmark("avg", 42u64, 1337u64, sigmoid_sat_u32, ITERATIONS);
    benchmark("min", 0u64, 0u64, sigmoid_sat_u32, ITERATIONS);
    benchmark("max", u32::MAX as u64, u32::MAX as u64, sigmoid_sat_u32, ITERATIONS);

    // exp2_u64_fixed tests  
    println!("\nexp2_u64_fixed:");
    benchmark("avg", 42u64, 1337u64, exp2_u64_fixed, ITERATIONS);
    benchmark("min", 0u64, 0u64, exp2_u64_fixed, ITERATIONS);
    benchmark("max", u64::MAX, 0u64, exp2_u64_fixed, ITERATIONS);
}

fn benchmark<F>(label: &str, val: u64, aux: u64, f: F, iterations: u64)
where
    F: Fn(u64, u64) -> u64,
{
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = f(black_box(val), black_box(aux));
    }
    let ns = start.elapsed().as_nanos() as f64 / iterations as f64;
    println!("  {}: {:.3} ns/iter", label, ns);
}
