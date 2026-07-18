use bcinr_cmca::fixed::NonNegativeFixed;
fn main() {
    let base: NonNegativeFixed = unreachable!();
    let _ = NonNegativeFixed { val: 1, ..base };
}
