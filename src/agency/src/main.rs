use agency::{
    Agency,
    runtime::{ScriptConfig, ScriptRuntime},
};
use agency_proto::{PortError, Result, client::Client};
use clap::{Parser, Subcommand};
use std::{path::PathBuf, sync::Arc};

#[derive(Parser)]
#[command(name = "agency", version, about = "Independent local Agency service")]
struct Args {
    #[arg(long)]
    root: PathBuf,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Start {
        #[arg(long)]
        script_config: Option<PathBuf>,
    },
    Serve {
        #[arg(long)]
        script_config: Option<PathBuf>,
    },
    Status,
    Stop,
}
#[tokio::main]
async fn main() {
    let args = Args::parse();
    if let Err(e) = run(args).await {
        println!("{}", serde_json::json!({"error":e}));
        std::process::exit(1);
    }
}
async fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Serve { script_config } => {
            let config = if let Some(path) = script_config {
                Some(serde_json::from_slice::<ScriptConfig>(&std::fs::read(
                    path,
                )?)?)
            } else {
                Agency::script_config(&args.root)?
            };
            let runtime: Arc<dyn agency::runtime::Runtime> = if let Some(config) = &config {
                Arc::new(ScriptRuntime::new(config.clone()))
            } else {
                Arc::new(Unconfigured)
            };
            let service = Agency::open(&args.root, runtime)?;
            if let Some(config) = &config {
                service.save_script_config(config)?;
            }
            agency::serve(service).await
        }
        Command::Start { script_config } => {
            if status(&args.root).await.is_ok() {
                println!(
                    "{}",
                    serde_json::json!({"ready":true,"already_running":true})
                );
                return Ok(());
            }
            std::fs::create_dir_all(&args.root)?;
            let mut command = std::process::Command::new(std::env::current_exe()?);
            command.arg("--root").arg(&args.root).arg("serve");
            if let Some(config) = script_config {
                command.arg("--script-config").arg(config);
            }
            command
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()?;
            for _ in 0..100 {
                if let Ok(value) = status(&args.root).await {
                    println!("{value}");
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(PortError::new(
                "AGENCY_NOT_READY",
                "local Agency did not become ready",
                "run_agency_serve",
            ))
        }
        Command::Status => {
            println!("{}", status(&args.root).await?);
            Ok(())
        }
        Command::Stop => {
            let key = Agency::bootstrap_key(&args.root)?;
            let value: serde_json::Value =
                Client::new(agency_proto::client::admin_endpoint(&args.root)?, key)
                    .call("shutdown", &serde_json::json!({}))
                    .await?;
            println!("{value}");
            Ok(())
        }
    }
}
async fn status(root: &std::path::Path) -> Result<serde_json::Value> {
    let key = Agency::bootstrap_key(root)?;
    Client::new(agency_proto::client::admin_endpoint(root)?, key)
        .call("catalog", &serde_json::json!({}))
        .await
}
struct Unconfigured;
impl agency::runtime::Runtime for Unconfigured {
    fn catalog(&self) -> Result<agency_proto::Catalog> {
        Ok(agency_proto::Catalog {
            professions: vec![],
            harnesses: vec![],
            skills: vec![],
        })
    }
    fn start(
        &self,
        _: &agency_proto::Sealed<agency_proto::ExecutionSpec>,
        _: &agency_proto::Sealed<agency_proto::context::Bundle>,
        _: &std::path::Path,
    ) -> Result<agency::runtime::Running> {
        Err(PortError::new(
            "RUNTIME_NOT_CONFIGURED",
            "no execution runtime installed",
            "configure_agency_runtime",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn command_schema_and_required_root_are_valid() {
        Args::command().debug_assert();
        assert!(Args::try_parse_from(["agency", "--root", "/tmp/fixture", "status"]).is_ok());
        assert!(Args::try_parse_from(["agency", "status"]).is_err());
    }
}
