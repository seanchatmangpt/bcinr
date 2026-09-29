#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub struct OcelEvent<'a>{pub event_type:&'a str,pub subject:&'a str,pub effect_id:&'a str,pub replay_id:&'a str}
impl<'a> OcelEvent<'a>{pub const fn receipt(subject:&'a str,effect_id:&'a str,replay_id:&'a str)->Self{Self{event_type:"sa2a.receipt",subject,effect_id,replay_id}}}
