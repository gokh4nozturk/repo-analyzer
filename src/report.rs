use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use std::fs::File;
use std::io::Write;

use crate::analyzer::RepositoryAnalysis;

#[derive(Serialize)]
struct JsonReport {
    repo_path: String,
    file_count: usize,
    language_stats: Vec<LanguageStat>,
    total_lines: usize,
    code_lines: usize,
    comment_lines: usize,
    blank_lines: usize,
    commit_count: usize,
    contributors: Vec<ContributorInfo>,
    last_activity: String,
    file_extensions: Vec<ExtensionStat>,
    avg_file_size: f64,
    largest_files: Vec<LargeFileInfo>,
    complexity_stats: ComplexityStats,
    file_age_stats: FileAgeStats,
    most_changed_files: Vec<FileChangeInfo>,
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

#[derive(Serialize)]
struct LargeFileInfo {
    path: String,
    size_bytes: usize,
    size_human: String,
}

#[derive(Serialize)]
struct ComplexityStats {
    avg_complexity: f64,
    max_complexity: usize,
    complex_files: Vec<ComplexFileInfo>,
    avg_function_length: f64,
    max_function_length: usize,
    long_functions: Vec<LongFunctionInfo>,
}

#[derive(Serialize)]
struct ComplexFileInfo {
    path: String,
    complexity: usize,
}

#[derive(Serialize)]
struct LongFunctionInfo {
    path: String,
    function_name: String,
    line_count: usize,
}

#[derive(Serialize)]
struct FileAgeStats {
    newest_files: Vec<FileAgeInfo>,
    oldest_files: Vec<FileAgeInfo>,
    most_modified_files: Vec<FileModificationInfo>,
}

#[derive(Serialize)]
struct FileAgeInfo {
    path: String,
    date: String,
}

#[derive(Serialize)]
struct FileModificationInfo {
    path: String,
    modification_count: usize,
}

#[derive(Serialize)]
struct FileChangeInfo {
    path: String,
    commit_count: usize,
    lines_added: usize,
    lines_removed: usize,
    change_frequency: f64,
    top_contributor: String,
    last_modified: String,
    avg_changes_per_commit: f64,
}

pub fn generate_report(
    analysis: &RepositoryAnalysis,
    format: String,
    top_contributors: usize,
) -> Result<()> {
    match format.to_lowercase().as_str() {
        "text" => generate_text_report(analysis, top_contributors),
        "json" => generate_json_report(analysis, top_contributors),
        "html" => generate_html_report(analysis, top_contributors),
        _ => {
            println!("Unsupported format: {}. Defaulting to text.", format);
            generate_text_report(analysis, top_contributors)
        }
    }
}

fn generate_text_report(analysis: &RepositoryAnalysis, top_contributors: usize) -> Result<()> {
    println!("\n{}", "Repository Analysis Report".yellow().bold());
    println!("{}", "=========================".yellow());

    println!("\n{}", "General Information:".cyan().bold());
    println!("Repository Path: {}", analysis.repo_path.display());
    println!("Total Files: {}", analysis.file_count);
    println!("Total Lines of Code: {}", analysis.total_lines);
    println!("Total Commits: {}", analysis.commit_count);
    println!("Last Activity: {}", analysis.last_activity);
    println!(
        "Average File Size: {:.2} KB",
        analysis.avg_file_size / 1024.0
    );

    println!("\n{}", "Language Statistics:".cyan().bold());
    let total_files = analysis.file_count as f64;
    let mut languages: Vec<(&String, &usize)> = analysis.language_stats.iter().collect();
    languages.sort_by(|(_, a), (_, b)| b.cmp(a));

    for (language, count) in languages {
        let percentage = (*count as f64 / total_files) * 100.0;
        println!("{}: {} files ({:.1}%)", language, count, percentage);
    }

    println!("\n{}", "File Extensions:".cyan().bold());
    let mut extensions: Vec<(&String, &usize)> = analysis.file_extensions.iter().collect();
    extensions.sort_by(|(_, a), (_, b)| b.cmp(a));

    for (ext, count) in extensions {
        let percentage = (*count as f64 / total_files) * 100.0;
        println!(".{}: {} files ({:.1}%)", ext, count, percentage);
    }

    println!("\n{}", "Largest Files:".cyan().bold());
    for (i, (path, size)) in analysis.largest_files.iter().enumerate().take(10) {
        println!(
            "{}. {} - {:.2} KB",
            i + 1,
            path.display(),
            *size as f64 / 1024.0
        );
    }

    println!("\n{}", "Top Contributors:".cyan().bold());
    for (i, contributor) in analysis
        .contributors
        .iter()
        .enumerate()
        .take(top_contributors)
    {
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

    println!("\n{}", "Most Changed Files:".cyan().bold());
    for (
        i,
        (
            path,
            commit_count,
            lines_added,
            lines_removed,
            change_frequency,
            top_contributor,
            _,
            _avg_changes,
        ),
    ) in analysis.most_changed_files.iter().enumerate().take(10)
    {
        println!(
            "{}. {} - {} commits, +{} -{}, {:.2} changes/month, by {}",
            i + 1,
            path.display(),
            commit_count,
            lines_added,
            lines_removed,
            change_frequency,
            top_contributor
        );
    }

    Ok(())
}

fn generate_json_report(analysis: &RepositoryAnalysis, top_contributors: usize) -> Result<()> {
    println!("Generating JSON report...");

    // Convert language stats to serializable format
    let language_stats: Vec<LanguageStat> = analysis
        .language_stats
        .iter()
        .map(|(language, count)| {
            let percentage = (*count as f64 / analysis.file_count as f64) * 100.0;
            LanguageStat {
                language: language.clone(),
                count: *count,
                percentage,
            }
        })
        .collect();

    // Convert file extensions to serializable format
    let file_extensions: Vec<ExtensionStat> = analysis
        .file_extensions
        .iter()
        .map(|(ext, count)| {
            let percentage = (*count as f64 / analysis.file_count as f64) * 100.0;
            ExtensionStat {
                extension: ext.clone(),
                count: *count,
                percentage,
            }
        })
        .collect();

    // Convert contributors to serializable format
    let contributors: Vec<ContributorInfo> = analysis
        .contributors
        .iter()
        .take(top_contributors)
        .map(|contributor| ContributorInfo {
            name: contributor.name.clone(),
            email: contributor.email.clone(),
            commit_count: contributor.commit_count,
            first_commit: contributor.first_commit.clone(),
            last_commit: contributor.last_commit.clone(),
        })
        .collect();

    // Convert largest files to serializable format
    let largest_files: Vec<LargeFileInfo> = analysis
        .largest_files
        .iter()
        .map(|(path, size)| LargeFileInfo {
            path: path.display().to_string(),
            size_bytes: *size,
            size_human: format!("{:.2} KB", *size as f64 / 1024.0),
        })
        .collect();

    // Convert complexity stats
    let complex_files: Vec<ComplexFileInfo> = analysis
        .complexity_stats
        .complex_files
        .iter()
        .map(|(path, complexity)| ComplexFileInfo {
            path: path.display().to_string(),
            complexity: *complexity,
        })
        .collect();

    let long_functions: Vec<LongFunctionInfo> = analysis
        .complexity_stats
        .long_functions
        .iter()
        .map(|(path, name, count)| LongFunctionInfo {
            path: path.display().to_string(),
            function_name: name.clone(),
            line_count: *count,
        })
        .collect();

    // Convert file age stats
    let newest_files: Vec<FileAgeInfo> = analysis
        .file_age_stats
        .newest_files
        .iter()
        .map(|(path, date)| FileAgeInfo {
            path: path.display().to_string(),
            date: date.clone(),
        })
        .collect();

    let oldest_files: Vec<FileAgeInfo> = analysis
        .file_age_stats
        .oldest_files
        .iter()
        .map(|(path, date)| FileAgeInfo {
            path: path.display().to_string(),
            date: date.clone(),
        })
        .collect();

    let most_modified_files: Vec<FileModificationInfo> = analysis
        .file_age_stats
        .most_modified_files
        .iter()
        .map(|(path, count)| FileModificationInfo {
            path: path.display().to_string(),
            modification_count: *count,
        })
        .collect();

    // Create most changed files info
    let most_changed_files: Vec<FileChangeInfo> = analysis
        .most_changed_files
        .iter()
        .map(
            |(
                path,
                commit_count,
                lines_added,
                lines_removed,
                change_frequency,
                top_contributor,
                last_modified,
                avg_changes_per_commit,
            )| {
                FileChangeInfo {
                    path: path.display().to_string(),
                    commit_count: *commit_count,
                    lines_added: *lines_added,
                    lines_removed: *lines_removed,
                    change_frequency: *change_frequency,
                    top_contributor: top_contributor.clone(),
                    last_modified: last_modified.clone(),
                    avg_changes_per_commit: *avg_changes_per_commit,
                }
            },
        )
        .collect();

    let complexity_stats = ComplexityStats {
        avg_complexity: analysis.complexity_stats.avg_complexity,
        max_complexity: analysis.complexity_stats.max_complexity,
        complex_files,
        avg_function_length: analysis.complexity_stats.avg_function_length,
        max_function_length: analysis.complexity_stats.max_function_length,
        long_functions,
    };

    let file_age_stats = FileAgeStats {
        newest_files,
        oldest_files,
        most_modified_files,
    };

    let report = JsonReport {
        repo_path: analysis.repo_path.display().to_string(),
        file_count: analysis.file_count,
        language_stats,
        total_lines: analysis.total_lines,
        code_lines: analysis.code_lines,
        comment_lines: analysis.comment_lines,
        blank_lines: analysis.blank_lines,
        commit_count: analysis.commit_count,
        contributors,
        last_activity: analysis.last_activity.clone(),
        file_extensions,
        avg_file_size: analysis.avg_file_size,
        largest_files,
        complexity_stats,
        file_age_stats,
        most_changed_files,
    };

    // Write to file
    let output_file = "repo_analysis.json";
    let file = File::create(output_file).context("Failed to create JSON report file")?;
    serde_json::to_writer_pretty(file, &report).context("Failed to write JSON report")?;

    println!("JSON report saved to {}", output_file);
    Ok(())
}

fn generate_html_report(analysis: &RepositoryAnalysis, top_contributors: usize) -> Result<()> {
    println!("Generating HTML report...");

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str("<title>Repository Analysis Report</title>\n");
    html.push_str("<style>\n");
    html.push_str("body { font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 1200px; margin: 0 auto; padding: 20px; }\n");
    html.push_str("h1, h2, h3 { color: #2c3e50; }\n");
    html.push_str("table { border-collapse: collapse; width: 100%; margin-bottom: 20px; }\n");
    html.push_str("th, td { text-align: left; padding: 12px; border-bottom: 1px solid #ddd; }\n");
    html.push_str("th { background-color: #f2f2f2; }\n");
    html.push_str("tr:hover { background-color: #f5f5f5; }\n");
    html.push_str(".card { background: white; border-radius: 5px; box-shadow: 0 2px 5px rgba(0,0,0,0.1); padding: 20px; margin-bottom: 20px; }\n");
    html.push_str(".stat { font-size: 24px; font-weight: bold; color: #3498db; }\n");
    html.push_str(".stat-label { font-size: 14px; color: #7f8c8d; }\n");
    html.push_str(
        ".stats-container { display: flex; flex-wrap: wrap; gap: 20px; margin-bottom: 20px; }\n",
    );
    html.push_str(".stat-box { flex: 1; min-width: 150px; background: #f8f9fa; padding: 15px; border-radius: 5px; text-align: center; }\n");
    html.push_str(".progress-bar { height: 10px; background: #ecf0f1; border-radius: 5px; margin-top: 5px; overflow: hidden; }\n");
    html.push_str(".progress-fill { height: 100%; background: #3498db; }\n");
    html.push_str(".tabs { display: flex; margin-bottom: 20px; }\n");
    html.push_str(".tab { padding: 10px 20px; cursor: pointer; background: #f2f2f2; border-radius: 5px 5px 0 0; }\n");
    html.push_str(".tab.active { background: #3498db; color: white; }\n");
    html.push_str(".tab-content { display: none; }\n");
    html.push_str(".tab-content.active { display: block; }\n");
    html.push_str("</style>\n");
    html.push_str("</head>\n<body>\n");

    // Header
    html.push_str(&format!(
        "<h1>Repository Analysis: {}</h1>\n",
        analysis.repo_path.display()
    ));

    // Overview stats
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>Overview</h2>\n");
    html.push_str("<div class=\"stats-container\">\n");

    html.push_str(&format!("<div class=\"stat-box\"><div class=\"stat\">{}</div><div class=\"stat-label\">Files</div></div>\n", 
        analysis.file_count));

    html.push_str(&format!("<div class=\"stat-box\"><div class=\"stat\">{}</div><div class=\"stat-label\">Lines of Code</div></div>\n", 
        analysis.total_lines));

    html.push_str(&format!("<div class=\"stat-box\"><div class=\"stat\">{}</div><div class=\"stat-label\">Commits</div></div>\n", 
        analysis.commit_count));

    html.push_str(&format!("<div class=\"stat-box\"><div class=\"stat\">{}</div><div class=\"stat-label\">Contributors</div></div>\n", 
        analysis.contributors.len()));

    html.push_str(&format!("<div class=\"stat-box\"><div class=\"stat\">{:.2}</div><div class=\"stat-label\">Avg Complexity</div></div>\n", 
        analysis.complexity_stats.avg_complexity));

    html.push_str("</div>\n"); // End stats-container
    html.push_str("</div>\n"); // End card

    // Language stats
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>Language Statistics</h2>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>Language</th><th>Files</th><th>Percentage</th></tr>\n");

    let mut languages: Vec<(&String, &usize)> = analysis.language_stats.iter().collect();
    languages.sort_by(|(_, a), (_, b)| b.cmp(a));

    for (language, count) in languages {
        let percentage = (*count as f64 / analysis.file_count as f64) * 100.0;
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{:.1}%</td></tr>\n",
            language, count, percentage
        ));
    }

    html.push_str("</table>\n");
    html.push_str("</div>\n"); // End card

    // Contributors
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>Top Contributors</h2>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>Name</th><th>Email</th><th>Commits</th><th>First Commit</th><th>Last Commit</th></tr>\n");

    for contributor in analysis.contributors.iter().take(top_contributors) {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            contributor.name,
            contributor.email,
            contributor.commit_count,
            contributor.first_commit,
            contributor.last_commit
        ));
    }

    html.push_str("</table>\n");
    html.push_str("</div>\n"); // End card

    // Code Complexity
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>Code Complexity</h2>\n");

    html.push_str("<h3>Most Complex Files</h3>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Complexity</th></tr>\n");

    for (path, complexity) in &analysis.complexity_stats.complex_files {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            path.display(),
            complexity
        ));
    }

    html.push_str("</table>\n");

    html.push_str("<h3>Longest Functions</h3>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Function</th><th>Lines</th></tr>\n");

    for (path, name, lines) in &analysis.complexity_stats.long_functions {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td></tr>\n",
            path.display(),
            name,
            lines
        ));
    }

    html.push_str("</table>\n");
    html.push_str("</div>\n"); // End card

    // File Age Statistics
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>File Age Statistics</h2>\n");

    html.push_str("<h3>Newest Files</h3>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Date</th></tr>\n");

    for (path, date) in &analysis.file_age_stats.newest_files {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            path.display(),
            date
        ));
    }

    html.push_str("</table>\n");

    html.push_str("<h3>Oldest Files</h3>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Date</th></tr>\n");

    for (path, date) in &analysis.file_age_stats.oldest_files {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            path.display(),
            date
        ));
    }

    html.push_str("</table>\n");

    html.push_str("<h3>Most Modified Files</h3>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Modifications</th></tr>\n");

    for (path, count) in &analysis.file_age_stats.most_modified_files {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td></tr>\n",
            path.display(),
            count
        ));
    }

    html.push_str("</table>\n");
    html.push_str("</div>\n"); // End card

    // Most Changed Files
    html.push_str("<div class=\"card\">\n");
    html.push_str("<h2>Most Changed Files</h2>\n");
    html.push_str("<table>\n");
    html.push_str("<tr><th>File</th><th>Commits</th><th>Lines Added</th><th>Lines Removed</th><th>Change Frequency</th><th>Top Contributor</th></tr>\n");

    for (
        path,
        commit_count,
        lines_added,
        lines_removed,
        change_frequency,
        top_contributor,
        _,
        _avg_changes,
    ) in &analysis.most_changed_files
    {
        html.push_str(&format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{:.2}</td><td>{}</td></tr>\n",
            path.display(),
            commit_count,
            lines_added,
            lines_removed,
            change_frequency,
            top_contributor
        ));
    }

    html.push_str("</table>\n");
    html.push_str("</div>\n"); // End card

    // Footer
    html.push_str("<div style=\"text-align: center; margin-top: 30px; color: #7f8c8d;\">\n");
    html.push_str("<p>Generated by Repository Analyzer</p>\n");
    html.push_str("</div>\n");

    html.push_str("</body>\n</html>");

    // Write to file
    let output_file = "repo_analysis.html";
    let mut file = File::create(output_file).context("Failed to create HTML report file")?;
    file.write_all(html.as_bytes())
        .context("Failed to write HTML report")?;

    println!("HTML report saved to {}", output_file);
    Ok(())
}
