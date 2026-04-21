#![feature(generic_const_exprs)]
#![allow(incomplete_features)]

use std::arch::asm;
use std::mem::{align_of, size_of};
use std::pin::Pin;

const WORDS: usize = 4096;
const BLOCK_BITS: usize = 262_144;
const REGION_BITS: usize = BLOCK_BITS * 2;

struct Work<const BITS: usize>
where
    [(); BITS / 64]:,
{
    words: [u64; BITS / 64],
}

#[repr(C, align(64))]
struct TruthBlock {
    words: [u64; WORDS],
}

#[repr(C, align(64))]
struct Scratchpad {
    words: [u64; WORDS],
}

#[repr(C, align(64))]
struct L1Region {
    truth: TruthBlock,
    scratch: Scratchpad,
}

impl L1Region {
    fn zeroed() -> Self {
        Self {
            truth: TruthBlock { words: [0; WORDS] },
            scratch: Scratchpad { words: [0; WORDS] },
        }
    }

    fn base_addr(&self) -> usize {
        self as *const L1Region as usize
    }

    fn truth_addr(&self) -> usize {
        &self.truth as *const TruthBlock as usize
    }

    fn scratch_addr(&self) -> usize {
        &self.scratch as *const Scratchpad as usize
    }
}

#[derive(Debug, Clone, Copy)]
struct L1Position {
    base: usize,
    truth: usize,
    scratch: usize,
    truth_offset: usize,
    scratch_offset: usize,
}

fn validate_l1_position(region: &L1Region) -> L1Position {
    assert_eq!(size_of::<TruthBlock>() * 8, BLOCK_BITS);
    assert_eq!(size_of::<Scratchpad>() * 8, BLOCK_BITS);
    assert_eq!(size_of::<L1Region>() * 8, REGION_BITS);

    assert_eq!(align_of::<TruthBlock>(), 64);
    assert_eq!(align_of::<Scratchpad>(), 64);
    assert_eq!(align_of::<L1Region>(), 64);

    let base = region.base_addr();
    let truth = region.truth_addr();
    let scratch = region.scratch_addr();

    assert_eq!(base % 64, 0);
    assert_eq!(truth % 64, 0);
    assert_eq!(scratch % 64, 0);

    let truth_offset = truth - base;
    let scratch_offset = scratch - base;

    assert_eq!(truth_offset, 0);
    assert_eq!(scratch_offset, size_of::<TruthBlock>());

    L1Position {
        base,
        truth,
        scratch,
        truth_offset,
        scratch_offset,
    }
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn asm_add_one(x: u64) -> u64 {
    let out: u64;

    unsafe {
        asm!(
            "lea {out}, [{x} + 1]",
            x = in(reg) x,
            out = lateout(reg) out,
            options(nomem, nostack, preserves_flags)
        );
    }

    out
}

#[cfg(not(target_arch = "x86_64"))]
#[inline(always)]
unsafe fn asm_add_one(x: u64) -> u64 {
    x + 1
}

fn main() {
    let _work: Work<512> = Work { words: [0; 8] };

    let region: Pin<Box<L1Region>> = Box::pin(L1Region::zeroed());

    let pos1 = validate_l1_position(&region);
    let pos2 = validate_l1_position(&region);

    assert_eq!(pos1.base, pos2.base);
    assert_eq!(pos1.truth, pos2.truth);
    assert_eq!(pos1.scratch, pos2.scratch);

    let asm_result = unsafe { asm_add_one(41) };
    assert_eq!(asm_result, 42);

    println!("nightly hello world passed");
    println!("generic_const_exprs passed");
    println!("pinned L1 position validated");
    println!("inline asm smoke passed");
    println!("base    = 0x{:x}", pos1.base);
    println!("truth   = 0x{:x} offset={}", pos1.truth, pos1.truth_offset);
    println!("scratch = 0x{:x} offset={}", pos1.scratch, pos1.scratch_offset);
}