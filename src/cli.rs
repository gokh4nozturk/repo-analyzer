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
    #[arg(short, long, required_unless_present = "remote_url")]
    pub repo_path: Option<PathBuf>,

    /// Output format (text, json, html)
    #[arg(short, long, default_value = "html")]
    pub output_format: String,

    /// Include detailed commit history
    #[arg(short, long, default_value = "false")]
    pub detailed_history: bool,

    /// Number of top contributors to show
    #[arg(short, long, default_value = "5")]
    pub top_contributors: usize,

    /// Clone and analyze a remote repository (provide URL)
    #[arg(short = 'u', long)]
    pub remote_url: Option<String>,

    /// Depth of commit history to analyze (0 for all)
    #[arg(long, default_value = "0")]
    pub history_depth: usize,
}
