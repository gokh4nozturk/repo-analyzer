use anyhow::Result;
use clap::Parser;
use colored::*;

mod analyzer;
mod cli;
mod git;
mod report;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", "Repository Analyzer".green().bold());
    println!(
        "Analyzing repository: {}",
        cli.repo_path.display().to_string().cyan()
    );

    let analysis = analyzer::analyze_repository(&cli.repo_path)?;
    report::generate_report(&analysis, cli.output_format)?;

    println!("{}", "Analysis complete!".green().bold());
    Ok(())
}
