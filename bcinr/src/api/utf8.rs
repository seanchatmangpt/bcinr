use bcinr_logic::::utf8;

/// Returns the length of the initial ASCII prefix in `bytes`.
#[inline(always)]
pub fn ascii_prefix_len(bytes: &[u8]) -> usize {
    utf8::ascii_prefix_len(bytes)
}
