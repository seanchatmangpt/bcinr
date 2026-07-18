use bcinr_cmca::shadow::ShadowExecutionReceipt;
fn main() {
    let _ = ShadowExecutionReceipt {
        admitted_proposal_digest: 0,
        current_mode_digest: 0,
        candidate_mode_digest: 0,
        round_identity: 0,
        comparison_value: 0,
        receipt_digest: 0,
    };
}
