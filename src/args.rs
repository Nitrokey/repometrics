use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Compare {
        #[arg(long)]
        metrics: String,
        baseline: PathBuf,
        test: PathBuf,
    },
    Generate {
        #[arg(long)]
        metrics: String,
        #[arg(long)]
        cache: bool,
        root: PathBuf,
    },
    Load {
        root: PathBuf,
        #[command(flatten)]
        rev: Rev,
    },
}

#[derive(Debug, clap::Args)]
#[group(multiple = false)]
pub struct Rev {
    #[arg(long)]
    pub rev: Option<String>,
    #[arg(long)]
    pub base: Option<String>,
}
