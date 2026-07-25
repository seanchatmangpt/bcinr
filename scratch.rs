#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpochReclamationRefusal {
    None = 0,
    EpochDesync = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpochReclamationResult {
    pub reclaim_mask: u8,
    pub refusal_code: u8,
}
