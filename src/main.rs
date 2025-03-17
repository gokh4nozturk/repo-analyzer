use anyhow::Result;
use clap::Parser;
use repo_analyzer::{analyzer, cli, report, s3};
use std::path::Path;
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let cli = cli::Cli::parse();

    // Determine repository path
    let repo_path = if let Some(path) = &cli.repo_path {
        path.clone()
    } else if let Some(url) = &cli.remote_url {
        // Clone remote repository to a temporary directory
        println!("Cloning repository from {}", url);
        let temp_dir =
            std::env::temp_dir().join(format!("repo-analyzer-{}", rand::random::<u32>()));
        git2::Repository::clone(url, &temp_dir)?;
        temp_dir
    } else {
        // This should not happen due to clap's required_unless_present
        return Err(anyhow::anyhow!("No repository path or URL provided"));
    };

    // Analyze repository
    let analysis = analyzer::analyze_repository(&repo_path, cli.history_depth)?;

    // Generate report
    let report_files =
        report::generate_report(&analysis, cli.output_format.clone(), cli.top_contributors)?;

    // Get the report file path based on the format
    let report_path = if let Some(custom_path) = &cli.output {
        custom_path.clone()
    } else {
        match cli.output_format.to_lowercase().as_str() {
            "json" => report_files
                .get("json")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "repo_analysis.json".to_string()),
            "html" => report_files
                .get("html")
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|| "repo_analysis.html".to_string()),
            _ => "repo_analysis.txt".to_string(),
        }
    };

    println!("Report generated: {}", report_path);

    // Upload report if requested
    if cli.upload {
        println!("Uploading report to cloud storage...");
        let url = s3::upload_to_s3(
            Path::new(&report_path),
            "repo-analyzer", // bucket name (not used with API)
            &report_path,    // key
            "eu-central-1",  // region (not used with API)
            true,            // always use API
        )
        .await?;

        println!("Report uploaded successfully!");
        println!("Access your report at: {}", url);
    }

    Ok(())
}
