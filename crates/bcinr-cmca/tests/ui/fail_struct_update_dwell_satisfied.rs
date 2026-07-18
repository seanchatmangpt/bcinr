use bcinr_cmca::certification::DwellSatisfied;
fn main() {
    let base: DwellSatisfied = unreachable!();
    let _ = DwellSatisfied { round_identity: 1, ..base };
}
