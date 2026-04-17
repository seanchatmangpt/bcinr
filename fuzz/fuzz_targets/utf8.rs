#![no_main]
use libfuzzer_sys::fuzz_target;
use bcinr_core::logic::utf8;

fuzz_target!(|data: &[u8]| {
    let _ = utf8::validate_utf8(data);
    let _ = utf8::count_codepoints(data);
    let _ = utf8::ascii_prefix_len(data);
    let _ = utf8::first_invalid_byte(data);
});
