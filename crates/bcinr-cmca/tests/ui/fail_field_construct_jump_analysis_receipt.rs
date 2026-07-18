use bcinr_cmca::jump::{JumpAnalysisReceipt, JumpKind};
fn main() {
    let _ = JumpAnalysisReceipt {
        kind: JumpKind::PolicyJump,
        shadow_receipt_digest: 0,
        magnitude: 0,
        analysis_digest: 0,
    };
}
