use bcinr_cmca::allocator::CertifiedLearning;
fn main() {
    let base: CertifiedLearning = unreachable!();
    let _ = CertifiedLearning { _sealed: (), ..base };
}
