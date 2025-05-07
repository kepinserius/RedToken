use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Honeytoken {
    pub id: Uuid,
    pub value: String,
    pub file_path: String,
    pub created_at: SystemTime,
    pub last_checked: Option<SystemTime>,
    pub is_triggered: bool,
}

impl Honeytoken {
    pub fn new(value: String, file_path: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            value,
            file_path,
            created_at: SystemTime::now(),
            last_checked: None,
            is_triggered: false,
        }
    }

    pub fn mark_as_triggered(&mut self) {
        self.is_triggered = true;
        self.last_checked = Some(SystemTime::now());
    }
}

#[async_trait::async_trait]
pub trait TokenRepository: Send + Sync {
    async fn save(&self, token: &Honeytoken) -> anyhow::Result<()>;
    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Honeytoken>>;
    async fn find_by_value(&self, value: &str) -> anyhow::Result<Option<Honeytoken>>;
    async fn find_all(&self) -> anyhow::Result<Vec<Honeytoken>>;
    async fn update(&self, token: &Honeytoken) -> anyhow::Result<()>;
}
