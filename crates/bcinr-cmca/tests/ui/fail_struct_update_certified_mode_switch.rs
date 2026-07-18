use bcinr_cmca::mode_switch::CertifiedModeSwitch;
fn main() {
    let base: CertifiedModeSwitch = unreachable!();
    let _ = CertifiedModeSwitch { target_mode_digest: 1, ..base };
}
