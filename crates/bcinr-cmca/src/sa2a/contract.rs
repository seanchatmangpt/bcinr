pub const CONTRACT: &str = "sa2a/replan-envelope/v1";
#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub enum RecoveryDecision { Reconcile, Replan, Terminal }
#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub struct Envelope<'a> { pub contract:&'a str, pub subject:&'a str, pub effect_id:&'a str, pub replay_id:&'a str, pub authority:&'a str }
impl<'a> Envelope<'a> { pub const fn powerless(subject:&'a str,effect_id:&'a str,replay_id:&'a str)->Self { Self{contract:CONTRACT,subject,effect_id,replay_id,authority:"none"} } }
