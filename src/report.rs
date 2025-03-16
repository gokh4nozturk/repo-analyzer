use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use crate::analyzer::RepositoryAnalysis;

#[derive(Serialize)]
struct JsonReport {
    repo_path: String,
    file_count: usize,
    language_stats: Vec<LanguageStat>,
    total_lines: usize,
    commit_count: usize,
    contributors: Vec<ContributorInfo>,
    last_activity: String,
    file_extensions: Vec<ExtensionStat>,
}

#[derive(Serialize)]
struct LanguageStat {
    language: String,
    count: usize,
    percentage: f64,
}

#[derive(Serialize)]
struct ExtensionStat {
    extension: String,
    count: usize,
    percentage: f64,
}

#[derive(Serialize)]
struct ContributorInfo {
    name: String,
    email: String,
    commit_count: usize,
    first_commit: String,
    last_commit: String,
}

pub fn generate_report(analysis: &RepositoryAnalysis, format: String) -> Result<()> {
    match format.to_lowercase().as_str() {
        "text" => generate_text_report(analysis),
        "json" => generate_json_report(analysis),
        "html" => generate_html_report(analysis),
        _ => {
            println!("Unsupported format: {}. Defaulting to text.", format);
            generate_text_report(analysis)
        }
    }
}

fn generate_text_report(analysis: &RepositoryAnalysis) -> Result<()> {
    println!("\n{}", "Repository Analysis Report".yellow().bold());
    println!("{}", "=========================".yellow());

    println!("\n{}", "General Information:".cyan().bold());
    println!("Repository Path: {}", analysis.repo_path.display());
    println!("Total Files: {}", analysis.file_count);
    println!("Total Lines of Code: {}", analysis.total_lines);
    println!("Total Commits: {}", analysis.commit_count);
    println!("Last Activity: {}", analysis.last_activity);

    println!("\n{}", "Language Statistics:".cyan().bold());
    let total_files = analysis.file_count as f64;
    for (language, count) in &analysis.language_stats {
        let percentage = (*count as f64 / total_files) * 100.0;
        println!("{}: {} files ({:.1}%)", language, count, percentage);
    }

    println!("\n{}", "File Extensions:".cyan().bold());
    for (ext, count) in &analysis.file_extensions {
        let percentage = (*count as f64 / total_files) * 100.0;
        println!(".{}: {} files ({:.1}%)", ext, count, percentage);
    }

    println!("\n{}", "Top Contributors:".cyan().bold());
    for (i, contributor) in analysis.contributors.iter().enumerate().take(5) {
        println!(
            "{}. {} <{}> - {} commits (first: {}, last: {})",
            i + 1,
            contributor.name,
            contributor.email,
            contributor.commit_count,
            contributor.first_commit,
            contributor.last_commit
        );
    }

    Ok(())
}

fn generate_json_report(analysis: &RepositoryAnalysis) -> Result<()> {
    let total_files = analysis.file_count as f64;

    let language_stats: Vec<LanguageStat> = analysis
        .language_stats
        .iter()
        .map(|(language, count)| {
            let percentage = (*count as f64 / total_files) * 100.0;
            LanguageStat {
                language: language.clone(),
                count: *count,
                percentage,
            }
        })
        .collect();

    let file_extensions: Vec<ExtensionStat> = analysis
        .file_extensions
        .iter()
        .map(|(ext, count)| {
            let percentage = (*count as f64 / total_files) * 100.0;
            ExtensionStat {
                extension: ext.clone(),
                count: *count,
                percentage,
            }
        })
        .collect();

    let contributors: Vec<ContributorInfo> = analysis
        .contributors
        .iter()
        .map(|c| ContributorInfo {
            name: c.name.clone(),
            email: c.email.clone(),
            commit_count: c.commit_count,
            first_commit: c.first_commit.clone(),
            last_commit: c.last_commit.clone(),
        })
        .collect();

    let report = JsonReport {
        repo_path: analysis.repo_path.display().to_string(),
        file_count: analysis.file_count,
        language_stats,
        total_lines: analysis.total_lines,
        commit_count: analysis.commit_count,
        contributors,
        last_activity: analysis.last_activity.clone(),
        file_extensions,
    };

    let json =
        serde_json::to_string_pretty(&report).context("Failed to serialize report to JSON")?;

    let output_path = Path::new("report.json");
    let mut file = File::create(output_path).context("Failed to create report file")?;
    file.write_all(json.as_bytes())
        .context("Failed to write to report file")?;

    println!("JSON report saved to: {}", output_path.display());

    Ok(())
}

fn generate_html_report(analysis: &RepositoryAnalysis) -> Result<()> {
    let total_files = analysis.file_count as f64;

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html lang=\"en\">\n");
    html.push_str("<head>\n");
    html.push_str("    <meta charset=\"UTF-8\">\n");
    html.push_str(
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n",
    );
    html.push_str("    <title>Repository Analysis Report</title>\n");
    html.push_str("    <style>\n");
    html.push_str("        body { font-family: Arial, sans-serif; margin: 0; padding: 20px; line-height: 1.6; }\n");
    html.push_str("        h1, h2 { color: #333; }\n");
    html.push_str("        .container { max-width: 1200px; margin: 0 auto; }\n");
    html.push_str("        .section { margin-bottom: 30px; }\n");
    html.push_str(
        "        table { width: 100%; border-collapse: collapse; margin-bottom: 20px; }\n",
    );
    html.push_str(
        "        th, td { padding: 12px; text-align: left; border-bottom: 1px solid #ddd; }\n",
    );
    html.push_str("        th { background-color: #f2f2f2; }\n");
    html.push_str(
        "        .bar-container { background-color: #f1f1f1; width: 100%; border-radius: 4px; }\n",
    );
    html.push_str(
        "        .bar { height: 20px; background-color: #4CAF50; border-radius: 4px; }\n",
    );
    html.push_str("    </style>\n");
    html.push_str("</head>\n");
    html.push_str("<body>\n");
    html.push_str("    <div class=\"container\">\n");
    html.push_str("        <h1>Repository Analysis Report</h1>\n");

    // General Information
    html.push_str("        <div class=\"section\">\n");
    html.push_str("            <h2>General Information</h2>\n");
    html.push_str("            <table>\n");
    html.push_str("                <tr><th>Repository Path</th><td>");
    html.push_str(&analysis.repo_path.display().to_string());
    html.push_str("</td></tr>\n");
    html.push_str("                <tr><th>Total Files</th><td>");
    html.push_str(&analysis.file_count.to_string());
    html.push_str("</td></tr>\n");
    html.push_str("                <tr><th>Total Lines of Code</th><td>");
    html.push_str(&analysis.total_lines.to_string());
    html.push_str("</td></tr>\n");
    html.push_str("                <tr><th>Total Commits</th><td>");
    html.push_str(&analysis.commit_count.to_string());
    html.push_str("</td></tr>\n");
    html.push_str("                <tr><th>Last Activity</th><td>");
    html.push_str(&analysis.last_activity);
    html.push_str("</td></tr>\n");
    html.push_str("            </table>\n");
    html.push_str("        </div>\n");

    // Language Statistics
    html.push_str("        <div class=\"section\">\n");
    html.push_str("            <h2>Language Statistics</h2>\n");
    html.push_str("            <table>\n");
    html.push_str("                <tr><th>Language</th><th>Files</th><th>Percentage</th><th>Distribution</th></tr>\n");

    for (language, count) in &analysis.language_stats {
        let percentage = (*count as f64 / total_files) * 100.0;
        html.push_str("                <tr>\n");
        html.push_str("                    <td>");
        html.push_str(language);
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&count.to_string());
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&format!("{:.1}%", percentage));
        html.push_str("</td>\n");
        html.push_str("                    <td>\n");
        html.push_str("                        <div class=\"bar-container\">\n");
        html.push_str("                            <div class=\"bar\" style=\"width: ");
        html.push_str(&format!("{:.1}%", percentage));
        html.push_str("\"></div>\n");
        html.push_str("                        </div>\n");
        html.push_str("                    </td>\n");
        html.push_str("                </tr>\n");
    }

    html.push_str("            </table>\n");
    html.push_str("        </div>\n");

    // File Extensions
    html.push_str("        <div class=\"section\">\n");
    html.push_str("            <h2>File Extensions</h2>\n");
    html.push_str("            <table>\n");
    html.push_str("                <tr><th>Extension</th><th>Files</th><th>Percentage</th></tr>\n");

    for (ext, count) in &analysis.file_extensions {
        let percentage = (*count as f64 / total_files) * 100.0;
        html.push_str("                <tr>\n");
        html.push_str("                    <td>.");
        html.push_str(ext);
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&count.to_string());
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&format!("{:.1}%", percentage));
        html.push_str("</td>\n");
        html.push_str("                </tr>\n");
    }

    html.push_str("            </table>\n");
    html.push_str("        </div>\n");

    // Contributors
    html.push_str("        <div class=\"section\">\n");
    html.push_str("            <h2>Top Contributors</h2>\n");
    html.push_str("            <table>\n");
    html.push_str("                <tr><th>Name</th><th>Email</th><th>Commits</th><th>First Commit</th><th>Last Commit</th></tr>\n");

    for contributor in analysis.contributors.iter().take(5) {
        html.push_str("                <tr>\n");
        html.push_str("                    <td>");
        html.push_str(&contributor.name);
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&contributor.email);
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&contributor.commit_count.to_string());
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&contributor.first_commit);
        html.push_str("</td>\n");
        html.push_str("                    <td>");
        html.push_str(&contributor.last_commit);
        html.push_str("</td>\n");
        html.push_str("                </tr>\n");
    }

    html.push_str("            </table>\n");
    html.push_str("        </div>\n");

    html.push_str("    </div>\n");
    html.push_str("</body>\n");
    html.push_str("</html>\n");

    let output_path = Path::new("report.html");
    let mut file = File::create(output_path).context("Failed to create HTML report file")?;
    file.write_all(html.as_bytes())
        .context("Failed to write to HTML report file")?;

    println!("HTML report saved to: {}", output_path.display());

    Ok(())
}
