#![no_main]
use libfuzzer_sys::fuzz_target;
use bcinr_core::logic::parse;

fuzz_target!(|data: &[u8]| {
    let _ = parse::parse_decimal_u64(data);
    let _ = parse::parse_hex_u32(data);
});
