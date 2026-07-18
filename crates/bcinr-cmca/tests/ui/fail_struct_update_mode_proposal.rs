use bcinr_cmca::proposal::ModeProposal;
fn main() {
    let base: ModeProposal = unreachable!();
    let _ = ModeProposal { round_identity: 1, ..base };
}
