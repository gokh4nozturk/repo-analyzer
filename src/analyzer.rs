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
    pub avg_file_size: f64,
    pub largest_files: Vec<(PathBuf, usize)>,
}

pub fn analyze_repository(repo_path: &Path, history_depth: usize) -> Result<RepositoryAnalysis> {
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
        avg_file_size: 0.0,
        largest_files: Vec::new(),
    };

    // Analyze files
    analyze_files(&repo_path, &mut analysis)?;

    // Analyze git history
    analyze_git_history(&repo_path, &mut analysis, history_depth)?;

    // Calculate average file size
    if analysis.file_count > 0 {
        let total_size: usize = analysis.largest_files.iter().map(|(_, size)| size).sum();
        analysis.avg_file_size = total_size as f64 / analysis.file_count as f64;
    }

    // Keep only top 10 largest files
    analysis.largest_files.sort_by(|(_, a), (_, b)| b.cmp(a));
    analysis.largest_files.truncate(10);

    Ok(analysis)
}

fn analyze_files(repo_path: &Path, analysis: &mut RepositoryAnalysis) -> Result<()> {
    println!("Analyzing files...");

    let ignore_patterns = vec![
        Regex::new(r"\.git/").unwrap(),
        Regex::new(r"node_modules/").unwrap(),
        Regex::new(r"target/").unwrap(),
        Regex::new(r"\.DS_Store").unwrap(),
        Regex::new(r"\.idea/").unwrap(),
        Regex::new(r"\.vscode/").unwrap(),
    ];

    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path(), &ignore_patterns))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        analysis.file_count += 1;

        // Get file size
        if let Ok(metadata) = entry.metadata() {
            let file_size = metadata.len() as usize;
            analysis
                .largest_files
                .push((entry.path().to_path_buf(), file_size));
        }

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
                    "jsx" => "React",
                    "tsx" => "React",
                    "py" => "Python",
                    "java" => "Java",
                    "c" | "h" => "C",
                    "cpp" | "hpp" => "C++",
                    "go" => "Go",
                    "rb" => "Ruby",
                    "php" => "PHP",
                    "html" => "HTML",
                    "css" => "CSS",
                    "scss" | "sass" => "SASS",
                    "md" => "Markdown",
                    "json" => "JSON",
                    "yml" | "yaml" => "YAML",
                    "toml" => "TOML",
                    "sh" | "bash" => "Shell",
                    "sql" => "SQL",
                    "swift" => "Swift",
                    "kt" | "kts" => "Kotlin",
                    "dart" => "Dart",
                    "ex" | "exs" => "Elixir",
                    "hs" => "Haskell",
                    "clj" => "Clojure",
                    "fs" => "F#",
                    "vue" => "Vue",
                    "svelte" => "Svelte",
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

fn analyze_git_history(
    repo_path: &Path,
    analysis: &mut RepositoryAnalysis,
    history_depth: usize,
) -> Result<()> {
    println!("Analyzing git history...");

    let (commit_count, contributors, last_activity) =
        git::analyze_git_repo(repo_path, history_depth)
            .context("Failed to analyze git repository")?;

    analysis.commit_count = commit_count;
    analysis.contributors = contributors;
    analysis.last_activity = last_activity;

    Ok(())
}

fn is_ignored(path: &Path, patterns: &[Regex]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| pattern.is_match(&path_str))
}
