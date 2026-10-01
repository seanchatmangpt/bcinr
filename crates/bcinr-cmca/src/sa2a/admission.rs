use super::contract::{Envelope, CONTRACT};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Refusal {
    Contract,
    Authority,
    EmptySubject,
    EmptyEffect,
    EmptyReplay,
}
pub fn admit(e: &Envelope<'_>) -> Result<(), Refusal> {
    if e.contract != CONTRACT {
        return Err(Refusal::Contract);
    }
    if e.authority != "none" {
        return Err(Refusal::Authority);
    }
    if e.subject.is_empty() {
        return Err(Refusal::EmptySubject);
    }
    if e.effect_id.is_empty() {
        return Err(Refusal::EmptyEffect);
    }
    if e.replay_id.is_empty() {
        return Err(Refusal::EmptyReplay);
    }
    Ok(())
}
