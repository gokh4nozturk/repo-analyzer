use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;

mod analyzer;
mod cli;
mod git;
mod report;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    println!("{}", "Repository Analyzer".green().bold());

    // Handle remote repository if URL is provided
    let repo_path = if let Some(url) = &cli.remote_url {
        clone_remote_repo(url)?
    } else if let Some(path) = &cli.repo_path {
        path.clone()
    } else {
        anyhow::bail!("Either --repo-path or --remote-url must be provided");
    };

    println!(
        "Analyzing repository: {}",
        repo_path.display().to_string().cyan()
    );

    let analysis = analyzer::analyze_repository(&repo_path, cli.history_depth)?;
    report::generate_report(&analysis, cli.output_format.clone(), cli.top_contributors)?;

    // Clean up temporary directory if we cloned a remote repo
    if cli.remote_url.is_some() {
        cleanup_temp_dir(&repo_path)?;
    }

    println!("{}", "Analysis complete!".green().bold());
    Ok(())
}

fn clone_remote_repo(url: &str) -> Result<PathBuf> {
    println!("Cloning remote repository: {}", url.cyan());

    // Create a temporary directory
    let temp_dir = std::env::temp_dir().join(format!("repo_analyzer_{}", rand::random::<u64>()));
    fs::create_dir_all(&temp_dir).context("Failed to create temporary directory")?;

    // Clone the repository
    git::clone_repository(url, &temp_dir).context("Failed to clone remote repository")?;

    println!(
        "Repository cloned to: {}",
        temp_dir.display().to_string().yellow()
    );
    Ok(temp_dir)
}

fn cleanup_temp_dir(dir: &PathBuf) -> Result<()> {
    println!("Cleaning up temporary directory...");
    fs::remove_dir_all(dir).context("Failed to remove temporary directory")?;
    Ok(())
}
