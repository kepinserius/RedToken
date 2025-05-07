use crate::core::{
    error::{RedTokenError, RedTokenResult},
    notification::{NotificationChannel, NotificationConfig, NotificationService},
    token::Honeytoken,
};
use async_trait::async_trait;
use log::{error, info};
use reqwest::{self, Client};
use serde_json::json;
use std::time::Duration;

// Composite notification service that can send to multiple channels
pub struct CompositeNotificationService {
    config: NotificationConfig,
    http_client: Client,
}

impl CompositeNotificationService {
    pub fn new(config: NotificationConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    async fn send_telegram(&self, webhook_url: &str, token: &Honeytoken) -> RedTokenResult<()> {
        let message = format!(
            "🚨 ALERT: Honeytoken triggered!\n\n\
            Token ID: {}\n\
            File Path: {}\n\
            Triggered: {}",
            token.id,
            token.file_path,
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        );

        let response = self
            .http_client
            .post(webhook_url)
            .json(&json!({
                "chat_id": "@redtoken_alerts", // This can be configured
                "text": message,
                "parse_mode": "HTML"
            }))
            .send()
            .await
            .map_err(|e| {
                RedTokenError::NotificationError(format!("Telegram request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(RedTokenError::NotificationError(format!(
                "Telegram API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        info!("Telegram notification sent for token {}", token.id);
        Ok(())
    }

    async fn send_discord(&self, webhook_url: &str, token: &Honeytoken) -> RedTokenResult<()> {
        let response = self
            .http_client
            .post(webhook_url)
            .json(&json!({
                "embeds": [{
                    "title": "🚨 Honeytoken Alert",
                    "description": "A honeytoken has been triggered!",
                    "color": 16711680, // Red
                    "fields": [
                        {
                            "name": "Token ID",
                            "value": token.id.to_string(),
                            "inline": true
                        },
                        {
                            "name": "File Path",
                            "value": token.file_path,
                            "inline": true
                        },
                        {
                            "name": "Triggered At",
                            "value": chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            "inline": false
                        }
                    ],
                    "footer": {
                        "text": "RedToken Intrusion Detection"
                    }
                }]
            }))
            .send()
            .await
            .map_err(|e| {
                RedTokenError::NotificationError(format!("Discord request failed: {}", e))
            })?;

        if !response.status().is_success() {
            return Err(RedTokenError::NotificationError(format!(
                "Discord API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            )));
        }

        info!("Discord notification sent for token {}", token.id);
        Ok(())
    }

    async fn send_email(
        &self,
        config: &NotificationChannel,
        token: &Honeytoken,
    ) -> RedTokenResult<()> {
        if let NotificationChannel::Email {
            smtp_server,
            from,
            to,
        } = config
        {
            // For simplicity in this version, we'll just log that we would send an email
            // In a real implementation, you would use lettre or another email library
            info!(
                "Would send email notification from {} to {} via {} for token {}",
                from, to, smtp_server, token.id
            );

            // Simplified implementation - just return success
            Ok(())
        } else {
            Err(RedTokenError::NotificationError(
                "Invalid email configuration".to_string(),
            ))
        }
    }
}

#[async_trait]
impl NotificationService for CompositeNotificationService {
    async fn send_alert(&self, token: &Honeytoken) -> anyhow::Result<()> {
        let mut success = false;

        for channel in &self.config.channels {
            match channel {
                NotificationChannel::Telegram { webhook_url } => {
                    if let Err(e) = self.send_telegram(webhook_url, token).await {
                        error!("Failed to send Telegram notification: {}", e);
                    } else {
                        success = true;
                    }
                }
                NotificationChannel::Discord { webhook_url } => {
                    if let Err(e) = self.send_discord(webhook_url, token).await {
                        error!("Failed to send Discord notification: {}", e);
                    } else {
                        success = true;
                    }
                }
                NotificationChannel::Email { .. } => {
                    if let Err(e) = self.send_email(channel, token).await {
                        error!("Failed to send Email notification: {}", e);
                    } else {
                        success = true;
                    }
                }
            }
        }

        if success {
            Ok(())
        } else {
            Err(anyhow::anyhow!("All notification channels failed"))
        }
    }
}
