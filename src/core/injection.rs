use crate::core::token::Honeytoken;

#[derive(Debug, Clone)]
pub enum FileType {
    Env,
    Json,
    Yaml,
    BashHistory,
    Custom(String),
}

#[async_trait::async_trait]
pub trait FileInjector: Send + Sync {
    async fn inject_token(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<()>;
    async fn verify_injection(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<bool>;
    async fn remove_token(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<()>;
}

pub struct InjectionConfig {
    pub file_type: FileType,
    pub backup_enabled: bool,
    pub injection_pattern: Option<String>,
    pub token_prefix: Option<String>,
    pub include_symbols: bool,
}
