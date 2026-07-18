use bcinr_cmca::stability::StabilityCandidate;
fn main() {
    let base: StabilityCandidate = unreachable!();
    let _ = StabilityCandidate { margin_delta: 1, ..base };
}
