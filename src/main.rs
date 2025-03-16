use anyhow::{Context, Result};
use clap::Parser;
use colored::*;
use std::fs;
use std::path::PathBuf;

mod analyzer;
mod cli;
mod config;
mod git;
mod report;
mod s3;

use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Load configuration
    let config = config::Config::load()?;

    println!("{}", "Repository Analyzer".green().bold());

    // Handle remote repository if URL is provided
    let repo_path = if let Some(url) = &cli.remote_url {
        clone_remote_repo(url)?
    } else if let Some(path) = &cli.repo_path {
        path.clone()
    } else {
        return Err(anyhow::anyhow!(
            "Either --repo-path or --remote-url must be provided"
        ));
    };

    println!(
        "Analyzing repository: {}",
        repo_path.display().to_string().cyan()
    );

    let analysis = analyzer::analyze_repository(&repo_path, cli.history_depth)?;
    let report_files =
        report::generate_report(&analysis, cli.output_format.clone(), cli.top_contributors)?;

    // Save report files locally
    let mut local_report_paths = Vec::new();
    for (format, file_path) in report_files.iter() {
        println!(
            "Report saved locally at: {}",
            file_path.display().to_string().cyan()
        );
        local_report_paths.push(file_path.clone());
    }

    // Upload reports to S3 if requested
    if cli.upload_to_s3 {
        for (format, file_path) in report_files.iter() {
            // Get repository name from path
            let repo_name = if repo_path.as_os_str() == "." {
                // If analyzing current directory, use the directory name
                std::env::current_dir()
                    .ok()
                    .and_then(|path| {
                        path.file_name()
                            .map(|name| name.to_string_lossy().to_string())
                    })
                    .unwrap_or_else(|| "repo-analyzer".to_string())
            } else if let Some(name) = repo_path.file_name() {
                name.to_string_lossy().to_string()
            } else {
                "unknown-repo".to_string()
            };

            let key = format!(
                "reports/{}-{}.{}",
                repo_name,
                chrono::Local::now().format("%Y%m%d%H%M%S"),
                format
            );

            println!("Uploading {} report to S3...", format);
            match s3::upload_to_s3(file_path, &config.aws.bucket, &key, &config.aws.region).await {
                Ok(url) => println!("Report available at: {}", url.cyan()),
                Err(e) => eprintln!("Failed to upload report to S3: {}", e),
            }
        }
    }

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
