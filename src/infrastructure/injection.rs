use crate::core::{
    error::{RedTokenError, RedTokenResult},
    injection::{FileInjector, FileType, InjectionConfig},
    token::Honeytoken,
};
use async_trait::async_trait;
use log::{debug, error, info};
use rand::{thread_rng, Rng};
use regex::Regex;
use serde_json::{self, json, Value};
use serde_yaml::{self, Value as YamlValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;

// File injection service that handles different file types
pub struct FileInjectionService {
    config: InjectionConfig,
}

impl FileInjectionService {
    pub fn new(config: InjectionConfig) -> Self {
        Self { config }
    }

    async fn backup_file(&self, file_path: &str) -> RedTokenResult<()> {
        if !self.config.backup_enabled {
            return Ok(());
        }

        let path = Path::new(file_path);
        let filename = path
            .file_name()
            .ok_or_else(|| RedTokenError::FileReadError {
                path: PathBuf::from(file_path),
                source: std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file path"),
            })?;

        let backup_dir = Path::new("backups");
        if !backup_dir.exists() {
            fs::create_dir_all(backup_dir)
                .await
                .map_err(|e| RedTokenError::FileWriteError {
                    path: backup_dir.to_path_buf(),
                    source: e,
                })?;
        }

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
        let backup_filename = format!("{}_{}", timestamp, filename.to_string_lossy());
        let backup_path = backup_dir.join(&backup_filename);

        fs::copy(file_path, &backup_path)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: backup_path.clone(),
                source: e,
            })?;

        debug!("Created backup of {} at {:?}", file_path, backup_path);
        Ok(())
    }

    // Generate a random token if not provided
    fn generate_token(&self, length: usize) -> String {
        let mut rng = thread_rng();
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        const SYMBOLS: &[u8] = b"!@#$%^&*()_+-=[]{}|;:,.<>?";

        let mut token = String::with_capacity(length);

        for _ in 0..length {
            let charset = if self.config.include_symbols && rng.gen_bool(0.2) {
                SYMBOLS
            } else {
                CHARSET
            };

            let idx = rng.gen_range(0..charset.len());
            token.push(charset[idx] as char);
        }

        // Add prefix if configured
        if let Some(ref prefix) = self.config.token_prefix {
            return format!("{}{}", prefix, token);
        }

        token
    }

    async fn inject_env(&self, file_path: &str, token: &Honeytoken) -> RedTokenResult<()> {
        // Backup the file if enabled
        self.backup_file(file_path).await?;

        // Read the file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| RedTokenError::FileReadError {
                    path: PathBuf::from(file_path),
                    source: e,
                })?;

        // Generate a random variable name if not specified
        let var_name = format!("API_TOKEN_{}", thread_rng().gen_range(100..999));

        // Append the token to the end of the file
        let new_content = format!(
            "{}\n\n# Added by RedToken\n{}={}\n",
            content.trim_end(),
            var_name,
            token.value
        );

        // Write the new content back to the file
        fs::write(file_path, new_content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: PathBuf::from(file_path),
                source: e,
            })?;

        info!("Injected token into .env file: {}", file_path);
        Ok(())
    }

    async fn inject_json(&self, file_path: &str, token: &Honeytoken) -> RedTokenResult<()> {
        // Backup the file if enabled
        self.backup_file(file_path).await?;

        // Read the file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| RedTokenError::FileReadError {
                    path: PathBuf::from(file_path),
                    source: e,
                })?;

        // Parse the JSON
        let mut json_value: Value = serde_json::from_str(&content)
            .map_err(|e| RedTokenError::InvalidFileFormat(format!("Invalid JSON: {}", e)))?;

        // Generate a random key name
        let key = format!("apiToken{}", thread_rng().gen_range(100..999));

        // Insert the token into the JSON
        if let Value::Object(ref mut map) = json_value {
            map.insert(key, Value::String(token.value.clone()));
        } else {
            // If the root is not an object, create an object with the token
            let mut map = serde_json::Map::new();
            map.insert(key, Value::String(token.value.clone()));
            json_value = Value::Object(map);
        }

        // Write the new content back to the file
        let new_content = serde_json::to_string_pretty(&json_value).map_err(|e| {
            RedTokenError::InvalidFileFormat(format!("Failed to serialize JSON: {}", e))
        })?;

        fs::write(file_path, new_content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: PathBuf::from(file_path),
                source: e,
            })?;

        info!("Injected token into JSON file: {}", file_path);
        Ok(())
    }

    async fn inject_yaml(&self, file_path: &str, token: &Honeytoken) -> RedTokenResult<()> {
        // Backup the file if enabled
        self.backup_file(file_path).await?;

        // Read the file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| RedTokenError::FileReadError {
                    path: PathBuf::from(file_path),
                    source: e,
                })?;

        // Parse the YAML
        let mut yaml_value: YamlValue = serde_yaml::from_str(&content)
            .map_err(|e| RedTokenError::InvalidFileFormat(format!("Invalid YAML: {}", e)))?;

        // Generate a random key name
        let key = format!("apiToken{}", thread_rng().gen_range(100..999));

        // Insert the token into the YAML
        if let YamlValue::Mapping(ref mut map) = yaml_value {
            map.insert(
                YamlValue::String(key),
                YamlValue::String(token.value.clone()),
            );
        } else {
            // If the root is not a mapping, create a mapping with the token
            let mut map = serde_yaml::Mapping::new();
            map.insert(
                YamlValue::String(key),
                YamlValue::String(token.value.clone()),
            );
            yaml_value = YamlValue::Mapping(map);
        }

        // Write the new content back to the file
        let new_content = serde_yaml::to_string(&yaml_value).map_err(|e| {
            RedTokenError::InvalidFileFormat(format!("Failed to serialize YAML: {}", e))
        })?;

        fs::write(file_path, new_content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: PathBuf::from(file_path),
                source: e,
            })?;

        info!("Injected token into YAML file: {}", file_path);
        Ok(())
    }

    async fn inject_bash_history(&self, file_path: &str, token: &Honeytoken) -> RedTokenResult<()> {
        // Backup the file if enabled
        self.backup_file(file_path).await?;

        // Read the file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| RedTokenError::FileReadError {
                    path: PathBuf::from(file_path),
                    source: e,
                })?;

        // Generate a command with the token
        let fake_commands = [
            format!(
                "curl -H 'Authorization: Bearer {}' https://api.example.com/v1/users",
                token.value
            ),
            format!(
                "aws s3 cp myfile.txt s3://mybucket --secret-key {}",
                token.value
            ),
            format!(
                "git clone https://{}@github.com/myorg/myrepo.git",
                token.value
            ),
            format!("export API_KEY={}", token.value),
        ];

        let command_idx = thread_rng().gen_range(0..fake_commands.len());
        let command = &fake_commands[command_idx];

        // Append the fake command to the end of the history file
        let new_content = format!("{}\n{}\n", content.trim_end(), command);

        // Write the new content back to the file
        fs::write(file_path, new_content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: PathBuf::from(file_path),
                source: e,
            })?;

        info!("Injected token into bash history: {}", file_path);
        Ok(())
    }
}

#[async_trait]
impl FileInjector for FileInjectionService {
    async fn inject_token(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<()> {
        // Infer file type from extension if not specified
        let file_type = &self.config.file_type;

        match file_type {
            FileType::Env => self.inject_env(file_path, token).await?,
            FileType::Json => self.inject_json(file_path, token).await?,
            FileType::Yaml => self.inject_yaml(file_path, token).await?,
            FileType::BashHistory => self.inject_bash_history(file_path, token).await?,
            FileType::Custom(_) => {
                // Use the injection pattern if provided
                if let Some(pattern) = &self.config.injection_pattern {
                    // Custom injection using the provided pattern
                    // Implement the logic...
                } else {
                    return Err(anyhow::anyhow!(
                        "Custom file type requires an injection pattern"
                    ));
                }
            }
        }

        Ok(())
    }

    async fn verify_injection(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<bool> {
        // Read the file content
        let content = fs::read_to_string(file_path).await?;

        // Check if the token is present in the file
        Ok(content.contains(&token.value))
    }

    async fn remove_token(&self, file_path: &str, token: &Honeytoken) -> anyhow::Result<()> {
        // Backup the file if enabled
        self.backup_file(file_path).await?;

        // Read the file content
        let content =
            fs::read_to_string(file_path)
                .await
                .map_err(|e| RedTokenError::FileReadError {
                    path: PathBuf::from(file_path),
                    source: e,
                })?;

        // Replace the token with a placeholder or remove it entirely
        // This is a simple implementation - for production, we would need more sophisticated
        // token removal logic based on file type

        let new_content = content.replace(&token.value, "[REDACTED]");

        // Write the new content back to the file
        fs::write(file_path, new_content)
            .await
            .map_err(|e| RedTokenError::FileWriteError {
                path: PathBuf::from(file_path),
                source: e,
            })?;

        info!("Removed token from file: {}", file_path);
        Ok(())
    }
}
