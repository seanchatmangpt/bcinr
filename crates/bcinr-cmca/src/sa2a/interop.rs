use super::{contract::Envelope,resource_envelope::{Allocation,ResourceEnvelope}};
#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub enum InteropRefusal{Authority,Resource}
pub fn allocate(e:&Envelope<'_>,budget:&ResourceEnvelope,want:&Allocation)->Result<Allocation,InteropRefusal>{if e.authority!="none"{return Err(InteropRefusal::Authority)} if !budget.admits(want){return Err(InteropRefusal::Resource)} Ok(*want)}
