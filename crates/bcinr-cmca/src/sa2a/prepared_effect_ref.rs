#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparedEffectRef<'a> {
    pub subject: &'a str,
    pub effect_id: &'a str,
    pub prepared_digest: &'a str,
}
impl<'a> PreparedEffectRef<'a> {
    pub const fn exact(subject: &'a str, effect_id: &'a str, digest: &'a str) -> Self {
        Self {
            subject,
            effect_id,
            prepared_digest: digest,
        }
    }
    pub const fn matches(&self, subject: &str, effect_id: &str, digest: &str) -> bool {
        self.subject.len() == subject.len()
            && self.effect_id.len() == effect_id.len()
            && self.prepared_digest.len() == digest.len()
    }
}
