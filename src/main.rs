mod config;
mod diff;
mod protocol;
mod snapshot;

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
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    match run(Cli::parse()).await {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::from(1)
        }
    }
}

async fn run(cli: Cli) -> Result<u8, String> {
    match cli.command {
        Commands::Baseline { config, output } | Commands::Replay { config, output } => {
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
        } => {
            let before = Snapshot::load(&baseline)?;
            let after = Snapshot::load(&current)?;
            emit_report(
                compare(&before, &after)?,
                json.as_deref(),
                markdown.as_deref(),
                fail_on,
            )
        }
        Commands::Check {
            config,
            baseline,
            json,
            markdown,
            fail_on,
        } => {
            let before = Snapshot::load(&baseline)?;
            let current = capture(&Config::load(&config)?).await?;
            emit_report(
                compare(&before, &current)?,
                json.as_deref(),
                markdown.as_deref(),
                fail_on,
            )
        }
    }
}

fn emit_report(
    report: Report,
    json: Option<&Path>,
    markdown: Option<&Path>,
    fail_on: FailOn,
) -> Result<u8, String> {
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
    println!("{rendered}");
    Ok(if report.should_fail(fail_on) { 2 } else { 0 })
}
