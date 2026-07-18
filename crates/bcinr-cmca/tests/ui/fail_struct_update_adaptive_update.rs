use bcinr_cmca::allocator::{AdaptiveUpdate, CertifiedLearning};
use core::marker::PhantomData;
fn main() {
    let base: AdaptiveUpdate<CertifiedLearning> = unreachable!();
    let _ = AdaptiveUpdate::<CertifiedLearning> { _mode: PhantomData, ..base };
}
