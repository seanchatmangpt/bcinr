use bcinr_cmca::fixed::{SignedFixed, NumericFaultSet};
fn main() {
    let _ = SignedFixed(0i32, NumericFaultSet::EMPTY);
}
