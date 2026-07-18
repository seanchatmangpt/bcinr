use bcinr_cmca::allocator::{AdaptiveUpdate, CertifiedLearning};
use core::marker::PhantomData;
fn main() {
    let _ = AdaptiveUpdate::<CertifiedLearning> { _mode: PhantomData };
}
