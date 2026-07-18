use bcinr_cmca::shadow::ShadowExecutionReceipt;
fn main() {
    let base: ShadowExecutionReceipt = unreachable!();
    let _ = ShadowExecutionReceipt { round_identity: 1, ..base };
}
