use anyhow::{Context, Result};
use regex::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::git;

#[derive(Debug)]
pub struct RepositoryAnalysis {
    pub repo_path: PathBuf,
    pub file_count: usize,
    pub language_stats: HashMap<String, usize>,
    pub total_lines: usize,
    pub commit_count: usize,
    pub contributors: Vec<git::Contributor>,
    pub last_activity: String,
    pub file_extensions: HashMap<String, usize>,
}

pub fn analyze_repository(repo_path: &Path) -> Result<RepositoryAnalysis> {
    let repo_path = repo_path.to_path_buf();

    // Initialize analysis structure
    let mut analysis = RepositoryAnalysis {
        repo_path: repo_path.clone(),
        file_count: 0,
        language_stats: HashMap::new(),
        total_lines: 0,
        commit_count: 0,
        contributors: Vec::new(),
        last_activity: String::new(),
        file_extensions: HashMap::new(),
    };

    // Analyze files
    analyze_files(&repo_path, &mut analysis)?;

    // Analyze git history
    analyze_git_history(&repo_path, &mut analysis)?;

    Ok(analysis)
}

fn analyze_files(repo_path: &Path, analysis: &mut RepositoryAnalysis) -> Result<()> {
    println!("Analyzing files...");

    let ignore_patterns = vec![
        Regex::new(r"\.git/").unwrap(),
        Regex::new(r"node_modules/").unwrap(),
        Regex::new(r"target/").unwrap(),
    ];

    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path(), &ignore_patterns))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        analysis.file_count += 1;

        // Get file extension
        if let Some(extension) = entry.path().extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext = ext_str.to_lowercase();
                *analysis.file_extensions.entry(ext.clone()).or_insert(0) += 1;

                // Map extensions to languages
                let language = match ext.as_str() {
                    "rs" => "Rust",
                    "js" => "JavaScript",
                    "ts" => "TypeScript",
                    "py" => "Python",
                    "java" => "Java",
                    "c" | "h" => "C",
                    "cpp" | "hpp" => "C++",
                    "go" => "Go",
                    "rb" => "Ruby",
                    "php" => "PHP",
                    "html" => "HTML",
                    "css" => "CSS",
                    "md" => "Markdown",
                    "json" => "JSON",
                    "yml" | "yaml" => "YAML",
                    "toml" => "TOML",
                    _ => "Other",
                };

                *analysis
                    .language_stats
                    .entry(language.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Count lines
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            analysis.total_lines += content.lines().count();
        }
    }

    Ok(())
}

fn analyze_git_history(repo_path: &Path, analysis: &mut RepositoryAnalysis) -> Result<()> {
    println!("Analyzing git history...");

    let (commit_count, contributors, last_activity) =
        git::analyze_git_repo(repo_path).context("Failed to analyze git repository")?;

    analysis.commit_count = commit_count;
    analysis.contributors = contributors;
    analysis.last_activity = last_activity;

    Ok(())
}

fn is_ignored(path: &Path, patterns: &[Regex]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| pattern.is_match(&path_str))
}
