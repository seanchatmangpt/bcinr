pub(crate) fn ascii_prefix_len(bytes: &[u8]) -> usize {
    bytes.iter().take_while(|&&b| b < 128).count()
}
