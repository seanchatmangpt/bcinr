use bcinr_cmca::jump::JumpAnalysisReceipt;
fn main() {
    let base: JumpAnalysisReceipt = unreachable!();
    let _ = JumpAnalysisReceipt { magnitude: 1, ..base };
}
