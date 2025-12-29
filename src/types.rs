use tokio::time::Instant;

#[derive(Debug, Clone)]
pub struct EntryChire {
    pub data: String,
    pub expire_at: Option<Instant>,
}

impl EntryChire {
    pub fn create(value: String) -> Self {
        Self {
            data: value,
            expire_at: None,
        }
    }
    pub fn set_expiration(&mut self, expire_at: Instant) {
        self.expire_at = Some(expire_at);
    }
}
