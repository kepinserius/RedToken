use crate::core::notification::NotificationChannel;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub storage: StorageConfig,
    pub web: WebConfig,
    pub notification: NotificationConfig,
    pub token: TokenConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_path: PathBuf,
    pub backup_enabled: bool,
    pub backup_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebConfig {
    pub port: u16,
    pub host: String,
    pub enable_ssl: bool,
    pub cert_path: Option<PathBuf>,
    pub key_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationConfig {
    pub channels: Vec<NotificationChannel>,
    pub rate_limit: Option<u32>, // Notifications per hour
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenConfig {
    pub token_length: usize,
    pub token_prefix: Option<String>,
    pub include_symbols: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            storage: StorageConfig {
                db_path: PathBuf::from("tokens.db"),
                backup_enabled: true,
                backup_path: Some(PathBuf::from("backups")),
            },
            web: WebConfig {
                port: 8080,
                host: "127.0.0.1".to_string(),
                enable_ssl: false,
                cert_path: None,
                key_path: None,
            },
            notification: NotificationConfig {
                channels: Vec::new(),
                rate_limit: Some(10),
            },
            token: TokenConfig {
                token_length: 32,
                token_prefix: Some("RT_".to_string()),
                include_symbols: true,
            },
        }
    }
}

impl AppConfig {
    pub fn load(path: &Path) -> Result<Self> {
        let config_str = fs::read_to_string(path)?;
        let config = serde_json::from_str(&config_str)?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let config_str = serde_json::to_string_pretty(self)?;
        fs::write(path, config_str)?;
        Ok(())
    }

    pub fn init_default(path: &Path) -> Result<Self> {
        let config = Self::default();
        config.save(path)?;
        Ok(config)
    }
}
