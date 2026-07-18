use bcinr_cmca::fixed::SignedFixed;
fn main() {
    let base: SignedFixed = unreachable!();
    let _ = SignedFixed { val: 1, ..base };
}
