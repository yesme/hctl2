use agency::{
    Agency,
    runtime::{ScriptConfig, ScriptRuntime},
};
use agency_proto::{PortError, Result, client::Client};
use clap::{Parser, Subcommand};
use std::{os::unix::process::CommandExt, path::PathBuf, sync::Arc};

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
#[cfg(target_os = "linux")]
#[path = "../linux_confine.rs"]
mod linux_confine;

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--confine") {
        #[cfg(target_os = "linux")]
        std::process::exit(linux_confine::run());
        #[cfg(not(target_os = "linux"))]
        {
            eprintln!("confine helper is only built for Linux");
            std::process::exit(1);
        }
    }
    let result = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime")
        .block_on(async {
            let args = Args::parse();
            run(args).await
        });
    if let Err(e) = result {
        println!("{}", serde_json::json!({"error":e}));
        std::process::exit(1);
    }
}
async fn run(args: Args) -> Result<()> {
    match args.command {
        Command::Serve { script_config } => {
            agency::confine::refuse_covered_credential_root(&args.root)?;
            let config = if let Some(path) = script_config {
                Some(serde_json::from_slice::<ScriptConfig>(&std::fs::read(
                    path,
                )?)?)
            } else {
                Agency::script_config(&args.root)?
            };
            let runtime: Arc<dyn agency::runtime::Runtime> = if let Some(config) = &config {
                Arc::new(ScriptRuntime::new(config.clone()))
            } else if let Some(install) = std::env::var_os("HCTL2_INSTALL_ROOT") {
                match catalog_installed_harness(std::path::Path::new(&install)) {
                    Ok(runtime) => {
                        if let Some(reason) = runtime.codex_skip() {
                            eprintln!("codex not cataloged: {reason}");
                        }
                        Arc::new(runtime)
                    }
                    Err(error) => {
                        eprintln!("harness not cataloged: {error}");
                        Arc::new(Unconfigured)
                    }
                }
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
            let log_path = args.root.join("serve.err");
            let log = Agency::start_log(&args.root)?;
            let mut command = std::process::Command::new(std::env::current_exe()?);
            command.arg("--root").arg(&args.root).arg("serve");
            if let Some(config) = script_config {
                command.arg("--script-config").arg(config);
            }
            let mut child = command
                .process_group(0)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(log)
                .spawn()?;
            // Herdr startup and two confined probes are bounded independently.
            // Do not give up before those probes can finish; retain their reasons.
            for _ in 0..1200 {
                if let Ok(value) = status(&args.root).await {
                    println!("{value}");
                    return Ok(());
                }
                if child.try_wait()?.is_some() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
            Err(PortError::new(
                "AGENCY_NOT_READY",
                format!(
                    "local Agency did not become ready; see {}",
                    log_path.display()
                ),
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
fn catalog_installed_harness(install: &std::path::Path) -> Result<agency::launch::InstalledHerdr> {
    let binary = agency::launch::installed_herdr(install)?;
    let claude = std::env::var_os("HCTL2_CLAUDE")
        .map(std::path::PathBuf::from)
        .or_else(claude_on_path)
        .ok_or_else(|| {
            PortError::new(
                "HARNESS_NOT_STARTED",
                "claude is not on PATH and HCTL2_CLAUDE is unset",
                "install_claude_code",
            )
        })?;
    agency::launch::InstalledHerdr::open(binary, &claude)
}

fn claude_on_path() -> Option<std::path::PathBuf> {
    let output = std::process::Command::new("/usr/bin/which")
        .arg("claude")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    let path = text.lines().next()?.trim();
    if path.is_empty() {
        None
    } else {
        Some(std::path::PathBuf::from(path))
    }
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
