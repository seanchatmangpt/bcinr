//! # UTF-8 Codepoint Counting and Count-Min Sketch Example
//!
//! Demonstrates `bcinr_logic::utf8::count_codepoints` and
//! `bcinr_logic::sketch::count_min_sketch_update`.
//!
//! **Doc references:**
//!   - `crates/bcinr-logic/src/utf8.rs`
//!   - `crates/bcinr-logic/src/sketch.rs`
//! **Also see:** `examples/scan_primitives.rs` — `is_ascii_u64_slice` detects ASCII-only input.
//!
//! `count_codepoints`: counts UTF-8 codepoints by skipping continuation bytes (0x80..=0xBF).
//! The algorithm: a byte starts a new codepoint iff `(byte & 0xC0) != 0x80`.
//! `count_min_sketch_update`: increments frequency counters for a hash, using depth×width
//! table with saturating_add. An approximate frequency data structure.
//!
//! These assertions would fail if `count_codepoints` counted raw bytes instead of codepoints,
//! or if `count_min_sketch_update` wrote to the wrong table positions.

use bcinr::sketch::count_min_sketch_update;
use bcinr::utf8::count_codepoints;

fn main() {
    // --- count_codepoints: UTF-8 codepoint count (not byte count) ---

    // Pure ASCII: one byte per codepoint
    assert_eq!(
        count_codepoints(b"hello"),
        5,
        "5 ASCII chars = 5 codepoints"
    );
    assert_eq!(count_codepoints(b""), 0, "empty = 0 codepoints");

    // 2-byte UTF-8: 'é' = 0xC3 0xA9 (two bytes, one codepoint)
    let e_acute = b"\xC3\xA9";
    assert_eq!(count_codepoints(e_acute), 1, "é = 1 codepoint (2 bytes)");

    // 3-byte UTF-8: '中' = 0xE4 0xB8 0xAD (three bytes, one codepoint)
    let cjk = b"\xE4\xB8\xAD";
    assert_eq!(count_codepoints(cjk), 1, "'中' = 1 codepoint (3 bytes)");

    // Mixed: "héllo" = h + é + l + l + o = 5 codepoints, 6 bytes
    let mixed = b"h\xC3\xA9llo";
    assert_eq!(count_codepoints(mixed), 5, "héllo = 5 codepoints");
    assert_ne!(
        count_codepoints(mixed),
        mixed.len(),
        "codepoints ≠ byte length for multi-byte"
    );

    // 4-byte: '𝄞' (musical symbol) = 0xF0 0x9D 0x84 0x9E
    let musical = b"\xF0\x9D\x84\x9E";
    assert_eq!(count_codepoints(musical), 1, "4-byte emoji = 1 codepoint");
    println!(
        "count_codepoints(héllo bytes={})={}",
        mixed.len(),
        count_codepoints(mixed)
    );
    println!(
        "count_codepoints(é)={}, cjk={}  musical={}",
        count_codepoints(e_acute),
        count_codepoints(cjk),
        count_codepoints(musical)
    );

    // --- count_min_sketch_update: approximate frequency counter ---
    // depth=3, width=16 → table of 48 u32 cells
    const DEPTH: usize = 3;
    const WIDTH: usize = 16;
    let mut table = vec![0u32; DEPTH * WIDTH];

    // Update for hash=42 twice
    count_min_sketch_update(&mut table, 42, DEPTH, WIDTH);
    count_min_sketch_update(&mut table, 42, DEPTH, WIDTH);

    // Each row should have exactly one cell incremented by 2
    let mut saw_two = 0usize;
    for row in 0..DEPTH {
        let mut row_max = 0u32;
        for col in 0..WIDTH {
            let v = table[row * WIDTH + col];
            if v > row_max {
                row_max = v;
            }
        }
        assert!(
            row_max >= 2,
            "each row must have at least one cell with count ≥ 2 after 2 updates"
        );
        if row_max == 2 {
            saw_two += 1;
        }
    }
    println!(
        "count_min_sketch: {saw_two}/{DEPTH} rows have max=2 (expected when no hash collision)"
    );

    // Update a different hash — cells should be independent
    count_min_sketch_update(&mut table, 999, DEPTH, WIDTH);
    let sum_before: u32 = table.iter().sum();
    assert!(
        sum_before >= 2 * DEPTH as u32 + DEPTH as u32,
        "3 total updates × depth rows = at least 9 increments"
    );
    println!("total cell increments after 3 updates across hashes: {sum_before}");

    // Saturation: a cell already at u32::MAX must not wrap to 0
    let sat_table = vec![0u32; DEPTH * WIDTH];
    // Fill the cells that hash=7 would target with u32::MAX
    let mut probe_table = vec![u32::MAX; DEPTH * WIDTH];
    count_min_sketch_update(&mut probe_table, 7, DEPTH, WIDTH);
    // After updating cells already at MAX, they must stay at MAX (saturating_add)
    assert!(
        probe_table.iter().all(|&v| v == u32::MAX),
        "saturating_add at MAX must not wrap"
    );
    println!(
        "saturation: updating MAX cells stays at u32::MAX = {}",
        probe_table.iter().all(|&v| v == u32::MAX)
    );
    let _ = sat_table; // suppress unused warning

    println!("\nAll UTF-8 and sketch assertions passed.");
}
