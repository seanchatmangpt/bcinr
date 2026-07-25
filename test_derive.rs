#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowlRunState {
    pub done_mask: u64,
    pub active_mask: u64,
    pub check_mask: u64,
    pub choice_taken: u64,
    pub loop_iters: [u8; 64],
    pub tick: u32,
    _pad: [u8; 4],
}
fn main() {}
