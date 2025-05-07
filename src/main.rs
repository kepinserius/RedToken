mod application;
mod core;
mod infrastructure;
mod interfaces;

use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::path::{PathBuf};
use std::sync::Arc;
use uuid::Uuid;

use application::config::AppConfig;
use application::service::RedTokenService;
use core::injection::{FileType, InjectionConfig};
use infrastructure::injection::FileInjectionService;
use infrastructure::notification::CompositeNotificationService;
use infrastructure::repository::{FileTokenRepository, InMemoryTokenRepository};
use interfaces::cli::{Cli, Commands};
use interfaces::web;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    // Parse command line arguments
    let cli = Cli::parse();

    // Load or create configuration
    let config_path = PathBuf::from("config.json");
    let config = if config_path.exists() {
        AppConfig::load(&config_path)?
    } else {
        info!("Creating default configuration at {:?}", config_path);
        AppConfig::init_default(&config_path)?
    };

    // Initialize repositories and services
    let token_repo = if config.storage.backup_enabled {
        Box::new(FileTokenRepository::new(&config.storage.db_path))
            as Box<dyn core::token::TokenRepository>
    } else {
        Box::new(InMemoryTokenRepository::new()) as Box<dyn core::token::TokenRepository>
    };

    // Buat NotificationConfig dari core menggunakan data config
    let notification_config = core::notification::NotificationConfig {
        channels: config.notification.channels.clone(),
        rate_limit: config.notification.rate_limit,
    };

    let notification_service = Box::new(CompositeNotificationService::new(notification_config));

    // Handle CLI commands
    match cli.command {
        Commands::Inject {
            file,
            value,
            file_type,
        } => {
            info!("Injecting token into {:?}", file);

            // Determine file type
            let file_type = match file_type.as_deref() {
                Some("env") => FileType::Env,
                Some("json") => FileType::Json,
                Some("yaml") => FileType::Yaml,
                Some("bash") => FileType::BashHistory,
                Some(custom) => FileType::Custom(custom.to_string()),
                None => {
                    // Auto-detect from extension
                    if let Some(ext) = file.extension() {
                        match ext.to_string_lossy().as_ref() {
                            "env" => FileType::Env,
                            "json" => FileType::Json,
                            "yaml" | "yml" => FileType::Yaml,
                            "history" => FileType::BashHistory,
                            _ => FileType::Custom("generic".to_string()),
                        }
                    } else {
                        FileType::Custom("generic".to_string())
                    }
                }
            };

            // Create injection config
            let injection_config = InjectionConfig {
                file_type,
                backup_enabled: config.storage.backup_enabled,
                injection_pattern: None,
                token_prefix: config.token.token_prefix.clone(),
                include_symbols: config.token.include_symbols,
            };

            let file_injector = Box::new(FileInjectionService::new(injection_config));

            // Create the main service
            let service = RedTokenService::new(token_repo, file_injector, notification_service);

            // Inject the token
            let token = service
                .inject_token(
                    file.to_string_lossy().as_ref(),
                    value.unwrap_or_else(|| format!("redtoken_{}", Uuid::new_v4())),
                )
                .await?;

            println!("Successfully injected token: {}", token.id);
            println!("Token value: {}", token.value);
            println!("File path: {}", token.file_path);
        }
        Commands::List => {
            info!("Listing all tokens");

            // Create a simple service just for listing
            let injection_config = InjectionConfig {
                file_type: FileType::Env,
                backup_enabled: false,
                injection_pattern: None,
                token_prefix: None,
                include_symbols: false,
            };

            let file_injector = Box::new(FileInjectionService::new(injection_config));

            let service = RedTokenService::new(token_repo, file_injector, notification_service);

            // List all tokens
            let tokens = service.list_tokens().await?;

            if tokens.is_empty() {
                println!("No tokens found.");
            } else {
                println!("Found {} tokens:", tokens.len());
                for token in tokens {
                    println!("ID: {}", token.id);
                    println!("Value: {}", token.value);
                    println!("File: {}", token.file_path);
                    println!(
                        "Triggered: {}",
                        if token.is_triggered { "Yes" } else { "No" }
                    );
                    println!("---");
                }
            }
        }
        Commands::Remove { id } => {
            info!("Removing token {}", id);

            let uuid = Uuid::parse_str(&id)?;

            // Create a simple service for removal
            let injection_config = InjectionConfig {
                file_type: FileType::Env, // Doesn't matter for removal
                backup_enabled: config.storage.backup_enabled,
                injection_pattern: None,
                token_prefix: None,
                include_symbols: false,
            };

            let file_injector = Box::new(FileInjectionService::new(injection_config));

            let service = RedTokenService::new(token_repo, file_injector, notification_service);

            // Remove the token
            service.remove_token(uuid).await?;

            println!("Token {} removed successfully.", id);
        }
        Commands::Serve { port } => {
            info!("Starting web server on port {}", port);

            // Create the service for the web server
            let injection_config = InjectionConfig {
                file_type: FileType::Env, // Default, but will be overridden per request
                backup_enabled: config.storage.backup_enabled,
                injection_pattern: None,
                token_prefix: config.token.token_prefix.clone(),
                include_symbols: config.token.include_symbols,
            };

            let file_injector = Box::new(FileInjectionService::new(injection_config));

            let service = Arc::new(RedTokenService::new(
                token_repo,
                file_injector,
                notification_service,
            ));

            // Start the web server
            web::start_server(service, port).await?;
        }
        Commands::Configure {
            telegram,
            discord,
            email,
        } => {
            info!("Configuring notification channels");

            let mut config = if config_path.exists() {
                AppConfig::load(&config_path)?
            } else {
                AppConfig::default()
            };

            let mut channels = Vec::new();

            if let Some(webhook_url) = telegram {
                channels.push(core::notification::NotificationChannel::Telegram { webhook_url });
                println!("Added Telegram notification channel");
            }

            if let Some(webhook_url) = discord {
                channels.push(core::notification::NotificationChannel::Discord { webhook_url });
                println!("Added Discord notification channel");
            }

            if let Some(email_config) = email {
                // Parse the email configuration
                // Format: "smtp://user:pass@server:port/from/to"
                if email_config.starts_with("smtp://") {
                    let parts: Vec<&str> = email_config.split('/').collect();
                    if parts.len() >= 4 {
                        let smtp_server = parts[2].to_string();
                        let from = parts[3].to_string();
                        let to = parts.get(4).unwrap_or(&"admin@example.com").to_string();

                        channels.push(core::notification::NotificationChannel::Email {
                            smtp_server,
                            from,
                            to,
                        });

                        println!("Added Email notification channel");
                    } else {
                        error!("Invalid email configuration format. Expected smtp://user:pass@server:port/from/to");
                    }
                } else {
                    error!("Invalid email configuration format. Expected smtp://user:pass@server:port/from/to");
                }
            }

            // Update the configuration
            config.notification.channels = channels;

            // Save the configuration
            config.save(&config_path)?;

            println!("Configuration saved to {:?}", config_path);
        }
    }

    Ok(())
}
