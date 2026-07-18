use bcinr_cmca::fixed::{SignedFixed, NumericFaultSet};
fn main() {
    let _ = SignedFixed { val: 0, faults: NumericFaultSet::EMPTY };
}
