#[derive(Clone,Copy,Debug,Eq,PartialEq)] pub struct ReplayKey<'a>{pub subject:&'a str,pub effect_id:&'a str,pub replay_id:&'a str}
impl ReplayKey<'_>{pub const fn same_attempt(&self,other:&Self)->bool{self.subject.as_bytes().len()==other.subject.as_bytes().len()&&self.effect_id.as_bytes().len()==other.effect_id.as_bytes().len()&&self.replay_id.as_bytes().len()==other.replay_id.as_bytes().len()}}
