use crate::core::{
    error::{RedTokenError, RedTokenResult},
    token::{Honeytoken, TokenRepository},
};
use async_trait::async_trait;
use log::{error, info};
use serde_json::{self, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::fs;
use uuid::Uuid;

// In-memory repository implementation
pub struct InMemoryTokenRepository {
    tokens: Arc<Mutex<HashMap<Uuid, Honeytoken>>>,
}

impl InMemoryTokenRepository {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TokenRepository for InMemoryTokenRepository {
    async fn save(&self, token: &Honeytoken) -> anyhow::Result<()> {
        let mut tokens = self
            .tokens
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        tokens.insert(token.id, token.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Honeytoken>> {
        let tokens = self
            .tokens
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(tokens.get(&id).cloned())
    }

    async fn find_by_value(&self, value: &str) -> anyhow::Result<Option<Honeytoken>> {
        let tokens = self
            .tokens
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(tokens.values().find(|t| t.value == value).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Honeytoken>> {
        let tokens = self
            .tokens
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        Ok(tokens.values().cloned().collect())
    }

    async fn update(&self, token: &Honeytoken) -> anyhow::Result<()> {
        let mut tokens = self
            .tokens
            .lock()
            .map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        tokens.insert(token.id, token.clone());
        Ok(())
    }
}

// File-based repository implementation
pub struct FileTokenRepository {
    db_path: PathBuf,
}

impl FileTokenRepository {
    pub fn new<P: AsRef<Path>>(db_path: P) -> Self {
        Self {
            db_path: db_path.as_ref().to_path_buf(),
        }
    }

    async fn read_db(&self) -> RedTokenResult<HashMap<Uuid, Honeytoken>> {
        if !self.db_path.exists() {
            return Ok(HashMap::new());
        }

        match fs::read_to_string(&self.db_path).await {
            Ok(content) => {
                if content.trim().is_empty() {
                    return Ok(HashMap::new());
                }

                match serde_json::from_str::<Vec<Honeytoken>>(&content) {
                    Ok(tokens) => {
                        let mut map = HashMap::new();
                        for token in tokens {
                            map.insert(token.id, token);
                        }
                        Ok(map)
                    }
                    Err(e) => Err(RedTokenError::DatabaseError(format!(
                        "Failed to parse database: {}",
                        e
                    ))),
                }
            }
            Err(e) => Err(RedTokenError::FileReadError {
                path: self.db_path.clone(),
                source: e,
            }),
        }
    }

    async fn write_db(&self, tokens: &HashMap<Uuid, Honeytoken>) -> RedTokenResult<()> {
        let tokens_vec: Vec<Honeytoken> = tokens.values().cloned().collect();
        let content = serde_json::to_string_pretty(&tokens_vec).map_err(|e| {
            RedTokenError::DatabaseError(format!("Failed to serialize database: {}", e))
        })?;

        // Create directory if it doesn't exist
        if let Some(parent) = self.db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)
                    .await
                    .map_err(|e| RedTokenError::FileWriteError {
                        path: parent.to_path_buf(),
                        source: e,
                    })?;
            }
        }

        fs::write(&self.db_path, content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: self.db_path.clone(),
                source: e,
            })?;

        Ok(())
    }
}

#[async_trait]
impl TokenRepository for FileTokenRepository {
    async fn save(&self, token: &Honeytoken) -> anyhow::Result<()> {
        let mut tokens = self.read_db().await?;
        tokens.insert(token.id, token.clone());
        self.write_db(&tokens).await?;
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> anyhow::Result<Option<Honeytoken>> {
        let tokens = self.read_db().await?;
        Ok(tokens.get(&id).cloned())
    }

    async fn find_by_value(&self, value: &str) -> anyhow::Result<Option<Honeytoken>> {
        let tokens = self.read_db().await?;
        Ok(tokens.values().find(|t| t.value == value).cloned())
    }

    async fn find_all(&self) -> anyhow::Result<Vec<Honeytoken>> {
        let tokens = self.read_db().await?;
        Ok(tokens.values().cloned().collect())
    }

    async fn update(&self, token: &Honeytoken) -> anyhow::Result<()> {
        let mut tokens = self.read_db().await?;
        tokens.insert(token.id, token.clone());
        self.write_db(&tokens).await?;
        Ok(())
    }
}
