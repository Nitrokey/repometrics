use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(long)]
    pub metrics: String,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Compare { baseline: PathBuf, test: PathBuf },
    Generate { root: PathBuf },
}
