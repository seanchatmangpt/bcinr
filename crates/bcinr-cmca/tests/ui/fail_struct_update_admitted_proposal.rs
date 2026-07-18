use bcinr_cmca::proposal::{AdmittedProposal, ModeProposal};
fn main() {
    let base: AdmittedProposal = unreachable!();
    let proposal: ModeProposal = unreachable!();
    let _ = AdmittedProposal { proposal, ..base };
}
