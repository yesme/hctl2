//! Foreground control daemon. `hctl2 start` launches this binary.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use control::Daemon;

#[derive(Parser)]
#[command(name = "hctl2-control", version, about = "HCTL2 control daemon")]
struct Args {
    /// Control data directory (governance records and the Unix socket).
    #[arg(long)]
    root: Option<PathBuf>,
    #[command(subcommand)]
    command: Option<Admin>,
}

#[derive(Subcommand)]
enum Admin {
    Github {
        #[command(subcommand)]
        command: Github,
    },
}
#[derive(Subcommand)]
enum Github {
    Login {
        #[arg(long, default_value = "github.com")]
        hostname: String,
        #[arg(long)]
        with_token: bool,
    },
    Status {
        #[arg(long, default_value = "github.com")]
        hostname: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(control::default_root);
    std::fs::create_dir_all(&root)?;
    if let Some(Admin::Github { command }) = args.command {
        let result = match command {
            Github::Login {
                hostname,
                with_token,
            } => control::github::login(&root, &hostname, with_token),
            Github::Status { hostname } => {
                println!("{}", control::github::status(&root, &hostname));
                return Ok(());
            }
        };
        match result {
            Ok(value) => println!("{value}"),
            Err(error) => {
                eprintln!(
                    "{}: {} ({})",
                    error.code, error.message, error.recovery_action
                );
                std::process::exit(1);
            }
        }
        return Ok(());
    }
    // Resolve the configured secret backend before serving: a configuration this
    // machine cannot satisfy (an explicit system-keyring with no keyring) is a
    // startup error, not something to discover on the first credential read.
    control::config::secret_store(&root).map_err(std::io::Error::other)?;
    Daemon::new(root).serve().await
}
