use bcinr_cmca::allocator::CertificateReceipt;
fn main() {
    let base: CertificateReceipt = unreachable!();
    let _ = CertificateReceipt { digest: 1, ..base };
}
