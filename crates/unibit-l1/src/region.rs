//! Resident L1 region.
//!
//! Generated from ontology.
//!
//! Law:
//!
//! ```text
//! truth stays resident
//! motion happens in scratch
//! ```

#![allow(dead_code)]

use core::mem::{align_of, size_of};

pub const U_TRUTH_BITS: usize = 262_144;
pub const U_SCRATCH_BITS: usize = 262_144;
pub const U_REGION_BITS: usize = U_TRUTH_BITS + U_SCRATCH_BITS;
pub const U_WORDS: usize = 4096;

#[repr(C, align(64))]
pub struct UTruthBlock {
    words: [u64; U_WORDS],
}

#[repr(C, align(64))]
pub struct UScratchpad {
    words: [u64; U_WORDS],
}

#[repr(C, align(64))]
pub struct UL1Region {
    truth: UTruthBlock,
    scratch: UScratchpad,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct UL1Position {
    pub base: usize,
    pub truth: usize,
    pub scratch: usize,
    pub truth_offset: usize,
    pub scratch_offset: usize,
}

impl UTruthBlock {
    pub const fn zeroed() -> Self {
        Self { words: [0; U_WORDS] }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const u64 {
        self.words.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u64 {
        self.words.as_mut_ptr()
    }
}

impl UScratchpad {
    pub const fn zeroed() -> Self {
        Self { words: [0; U_WORDS] }
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const u64 {
        self.words.as_ptr()
    }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut u64 {
        self.words.as_mut_ptr()
    }
}

impl UL1Region {
    pub const fn zeroed() -> Self {
        Self {
            truth: UTruthBlock::zeroed(),
            scratch: UScratchpad::zeroed(),
        }
    }

    #[inline(always)]
    pub fn truth(&self) -> &UTruthBlock {
        &self.truth
    }

    #[inline(always)]
    pub fn scratch(&self) -> &UScratchpad {
        &self.scratch
    }

    #[inline(always)]
    pub fn base_addr(&self) -> usize {
        self as *const UL1Region as usize
    }

    #[inline(always)]
    pub fn truth_addr(&self) -> usize {
        &self.truth as *const UTruthBlock as usize
    }

    #[inline(always)]
    pub fn scratch_addr(&self) -> usize {
        &self.scratch as *const UScratchpad as usize
    }

    pub fn validate_position(&self) -> UL1Position {
        assert_eq!(size_of::<UTruthBlock>() * 8, U_TRUTH_BITS);
        assert_eq!(size_of::<UScratchpad>() * 8, U_SCRATCH_BITS);
        assert_eq!(size_of::<UL1Region>() * 8, U_REGION_BITS);

        assert_eq!(align_of::<UTruthBlock>(), 64);
        assert_eq!(align_of::<UScratchpad>(), 64);
        assert_eq!(align_of::<UL1Region>(), 64);

        let base = self.base_addr();
        let truth = self.truth_addr();
        let scratch = self.scratch_addr();

        assert_eq!(base % 64, 0);
        assert_eq!(truth % 64, 0);
        assert_eq!(scratch % 64, 0);

        let truth_offset = truth - base;
        let scratch_offset = scratch - base;

        assert_eq!(truth_offset, 0);
        assert_eq!(scratch_offset * 8, U_TRUTH_BITS);

        UL1Position {
            base,
            truth,
            scratch,
            truth_offset,
            scratch_offset,
        }
    }
}