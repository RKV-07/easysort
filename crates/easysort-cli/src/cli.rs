use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "easysort",
    version,
    about = "Turns your Downloads folder into a clean, typed archive"
)]
pub struct Cli {
    /// Folder to organize (TUI mode). Defaults to ~/Downloads.
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Non-interactive sort (cron-friendly).
    Sort {
        /// Folder(s) to organize. Sorting rules apply to each path.
        #[arg(required = true, num_args = 1..)]
        paths: Vec<PathBuf>,
        /// Actually move files (default: dry run).
        #[arg(long)]
        run: bool,
        /// Sort by date instead of type.
        #[arg(long)]
        by_date: bool,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Undo the last applied batch from a manifest.
    Undo {
        /// Path to manifest file.
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
}
