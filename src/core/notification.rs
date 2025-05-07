use crate::core::token::Honeytoken;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    Telegram {
        webhook_url: String,
    },
    Discord {
        webhook_url: String,
    },
    Email {
        smtp_server: String,
        from: String,
        to: String,
    },
}

#[async_trait::async_trait]
pub trait NotificationService: Send + Sync {
    async fn send_alert(&self, token: &Honeytoken) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub channels: Vec<NotificationChannel>,
    pub rate_limit: Option<u32>, // Notifications per hour
}
