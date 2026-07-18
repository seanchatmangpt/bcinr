use bcinr_cmca::fixed::{NonNegativeFixed, NumericFaultSet};
fn main() {
    let _ = NonNegativeFixed { val: 0, faults: NumericFaultSet::EMPTY };
}
