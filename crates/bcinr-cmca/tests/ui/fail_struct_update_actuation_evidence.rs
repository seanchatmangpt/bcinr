use bcinr_cmca::mode_switch::ActuationEvidence;
fn main() {
    let base: ActuationEvidence = unreachable!();
    let _ = ActuationEvidence { round_identity: 1, ..base };
}
