mod list;
#[cfg(target_family = "unix")]
mod dump;
#[cfg(target_family = "windows")]
mod apply;

use clap::{Parser, Subcommand};
use eyre::bail;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Subcommand)]
pub(crate) enum Commands {
    Dump,
    List,
    Apply {
        adapter: String,
        device: String
    }
}

pub(super) fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    exec_cli(cli)
}

#[cfg(target_family = "unix")]
fn exec_cli(cli: Cli) -> eyre::Result<()> {
    match cli.command {
        Commands::Dump => dump::main(),
        Commands::List => list::main(),
        Commands::Apply { .. } => unsupported_cmd()
    }
}

#[cfg(target_family = "windows")]
fn exec_cli(cli: Cli) -> eyre::Result<()> {
    match cli.command {
        Commands::Dump => unsupported_cmd(),
        Commands::List => list::main(),
        Commands::Apply { adapter, device } => apply::main(&adapter, &device)
    }
}

fn unsupported_cmd() -> eyre::Result<()> {
    bail!("this command is not supported on the current platform")
}