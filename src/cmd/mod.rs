mod dump;
mod list;
mod apply;

use clap::{Parser, Subcommand};

pub const DUMP_FILE: &str = "dump.json";

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
    Apply
}

pub(super) fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Dump => dump::main()?,
        Commands::List => list::main()?,
        Commands::Apply => apply::main()?
    }

    Ok(())
}
