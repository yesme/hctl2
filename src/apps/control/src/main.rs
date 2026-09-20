//! Foreground control daemon. `hctl2 start` launches this binary.

#![forbid(unsafe_code)]

use std::path::PathBuf;

use clap::Parser;
use control::Daemon;

#[derive(Parser)]
#[command(name = "hctl2-control", version, about = "HCTL2 control daemon")]
struct Args {
    /// Control data directory (governance records and the Unix socket).
    #[arg(long)]
    root: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();
    let root = args.root.unwrap_or_else(control::default_root);
    std::fs::create_dir_all(&root)?;
    Daemon::new(root).serve().await
}
