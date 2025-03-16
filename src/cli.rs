use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "repo-analyzer",
    about = "A tool to analyze GitHub repositories",
    version,
    author
)]
pub struct Cli {
    /// Path to the repository to analyze
    #[arg(short, long)]
    pub repo_path: PathBuf,

    /// Output format (text, json, html)
    #[arg(short, long, default_value = "text")]
    pub output_format: String,

    /// Include detailed commit history
    #[arg(short, long, default_value = "false")]
    pub detailed_history: bool,

    /// Number of top contributors to show
    #[arg(short, long, default_value = "5")]
    pub top_contributors: usize,
}
