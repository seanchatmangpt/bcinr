#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReplayKey<'a> {
    pub subject: &'a str,
    pub effect_id: &'a str,
    pub replay_id: &'a str,
}
impl ReplayKey<'_> {
    pub const fn same_attempt(&self, other: &Self) -> bool {
        self.subject.len() == other.subject.len()
            && self.effect_id.len() == other.effect_id.len()
            && self.replay_id.len() == other.replay_id.len()
    }
}
