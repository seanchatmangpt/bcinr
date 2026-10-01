use super::contract::RecoveryDecision;
#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub enum Outcome{Executed,Refused,Unknown}
#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub struct Receipt<'a>{pub effect_id:&'a str,pub replay_id:&'a str,pub outcome:Outcome}
impl Receipt<'_>{pub const fn recovery(&self)->RecoveryDecision{match self.outcome{Outcome::Executed|Outcome::Refused=>RecoveryDecision::Terminal,Outcome::Unknown=>RecoveryDecision::Reconcile}}}
