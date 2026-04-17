#![no_std]
#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(all(test, feature = "std"))]
extern crate std;

pub mod bitset;
pub mod dfa;
pub mod exec;
pub mod fix;
pub mod int;
pub mod mask;
pub mod mem;
pub mod network;
pub mod parse;
pub mod reduce;
pub mod scan;
pub mod simd;
pub mod sketch;
pub mod swar;
pub mod utf8;

pub trait Branchless: Sized + Copy {}
impl Branchless for u8 {}
impl Branchless for u32 {}
impl Branchless for u64 {}
impl Branchless for i32 {}
impl Branchless for i64 {}
