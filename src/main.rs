use mcp_regress::{config, diff, paths::ensure_separate, policy::Policy, snapshot};

use clap::{Parser, Subcommand};
use config::Config;
use diff::{compare, FailOn, Report};
use snapshot::{capture, write_file, Snapshot};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
};

#[derive(Parser)]
#[command(
    name = "mcp-regress",
    version,
    about = "Record and compare MCP server behavior in CI"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate a fixture and optional policy without launching a server.
    Validate {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    /// Record a trusted server version as the baseline.
    Baseline {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Replay configured cases against the current server.
    Replay {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(short, long)]
        output: PathBuf,
    },
    /// Compare two saved snapshots.
    Diff {
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        current: PathBuf,
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(long)]
        markdown: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "breaking")]
        fail_on: FailOn,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
    /// Replay and compare with a saved baseline in one CI command.
    Check {
        #[arg(short, long)]
        config: PathBuf,
        #[arg(long)]
        baseline: PathBuf,
        #[arg(long)]
        json: Option<PathBuf>,
        #[arg(long)]
        markdown: Option<PathBuf>,
        #[arg(long, value_enum, default_value = "breaking")]
        fail_on: FailOn,
        #[arg(long)]
        policy: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = tokio::select! {
        result = run(cli) => result,
        _ = tokio::signal::ctrl_c() => Err("run cancelled".into()),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

async fn run(cli: Cli) -> Result<u8, String> {
    match cli.command {
        Commands::Validate { config, policy } => {
            Config::load(&config)?;
            if let Some(path) = policy {
                Policy::load(&path)?.apply(&mut Report {
                    format_version: 3,
                    changes: Vec::new(),
                })?;
            }
            println!("Configuration valid; no server launched");
            Ok(0)
        }
        Commands::Baseline { config, output } | Commands::Replay { config, output } => {
            ensure_separate(&[&config], &[&output])?;
            let config = Config::load(&config)?;
            let snapshot = capture(&config).await?;
            snapshot.save(&output)?;
            println!(
                "Saved {} tools and {} cases to {}",
                snapshot.tools.len(),
                snapshot.cases.len(),
                output.display()
            );
            Ok(0)
        }
        Commands::Diff {
            baseline,
            current,
            json,
            markdown,
            fail_on,
            policy,
        } => {
            let mut inputs = vec![baseline.as_path()];
            if let Some(path) = &policy {
                inputs.push(path);
            }
            let loaded_policy = policy.as_deref().map(Policy::load).transpose()?;
            if let Some(p) = &loaded_policy {
                p.apply(&mut Report {
                    format_version: 3,
                    changes: Vec::new(),
                })?;
            }
            let outputs: Vec<_> = json
                .iter()
                .chain(markdown.iter())
                .map(PathBuf::as_path)
                .collect();
            inputs.push(current.as_path());
            ensure_separate(&inputs, &outputs)?;
            let before = Snapshot::load(&baseline)?;
            let after = Snapshot::load(&current)?;
            emit_report(
                compare(&before, &after)?,
                json.as_deref(),
                markdown.as_deref(),
                fail_on,
                loaded_policy.as_ref(),
            )
        }
        Commands::Check {
            config,
            baseline,
            json,
            markdown,
            fail_on,
            policy,
        } => {
            let mut inputs = vec![baseline.as_path()];
            if let Some(path) = &policy {
                inputs.push(path);
            }
            let loaded_policy = policy.as_deref().map(Policy::load).transpose()?;
            if let Some(p) = &loaded_policy {
                p.apply(&mut Report {
                    format_version: 3,
                    changes: Vec::new(),
                })?;
            }
            let outputs: Vec<_> = json
                .iter()
                .chain(markdown.iter())
                .map(PathBuf::as_path)
                .collect();
            inputs.push(config.as_path());
            ensure_separate(&inputs, &outputs)?;
            let before = Snapshot::load(&baseline)?;
            let current = capture(&Config::load(&config)?).await?;
            emit_report(
                compare(&before, &current)?,
                json.as_deref(),
                markdown.as_deref(),
                fail_on,
                loaded_policy.as_ref(),
            )
        }
    }
}

fn emit_report(
    mut report: Report,
    json: Option<&Path>,
    markdown: Option<&Path>,
    fail_on: FailOn,
    policy: Option<&Policy>,
) -> Result<u8, String> {
    if let Some(policy) = policy {
        policy.apply(&mut report)?;
    }
    if let Some(path) = json {
        write_file(
            path,
            &serde_json::to_vec_pretty(&report).map_err(|e| e.to_string())?,
        )?;
    }
    let rendered = report.markdown();
    if let Some(path) = markdown {
        write_file(path, rendered.as_bytes())?;
    }
    println!(
        "{} change(s); {}",
        report.changes.len(),
        if report.should_fail(fail_on) {
            "policy failed"
        } else {
            "passed"
        }
    );
    Ok(if report.should_fail(fail_on) { 2 } else { 0 })
}
