use bcinr_cmca::fixed::{NonNegativeFixed, NumericFaultSet};
fn main() {
    let _ = NonNegativeFixed(0u32, NumericFaultSet::EMPTY);
}
