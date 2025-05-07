use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Inject a honeytoken into a file
    Inject {
        /// Path to the target file
        #[arg(short, long)]
        file: PathBuf,

        /// Custom token value (optional)
        #[arg(short, long)]
        value: Option<String>,

        /// File type (env, json, yaml, bash)
        #[arg(short, long)]
        file_type: Option<String>,
    },

    /// List all injected tokens
    List,

    /// Remove a token
    Remove {
        /// Token ID to remove
        #[arg(short, long)]
        id: String,
    },

    /// Start the web server for token monitoring
    Serve {
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    /// Configure notification channels
    Configure {
        /// Telegram webhook URL
        #[arg(long)]
        telegram: Option<String>,

        /// Discord webhook URL
        #[arg(long)]
        discord: Option<String>,

        /// Email configuration (format: "smtp://user:pass@server:port")
        #[arg(long)]
        email: Option<String>,
    },
}
