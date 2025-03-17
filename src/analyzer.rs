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
    pub code_lines: usize,
    pub comment_lines: usize,
    pub blank_lines: usize,
    pub commit_count: usize,
    pub contributors: Vec<git::Contributor>,
    pub last_activity: String,
    pub file_extensions: HashMap<String, usize>,
    pub avg_file_size: f64,
    pub largest_files: Vec<(PathBuf, usize)>,
    pub complexity_stats: ComplexityStats,
    pub file_age_stats: FileAgeStats,
    pub duplicate_code: Vec<DuplicateCode>,
    pub most_changed_files: Vec<(PathBuf, usize, usize, usize, f64, String, String, f64)>,
}

#[derive(Debug)]
pub struct ComplexityStats {
    pub avg_complexity: f64,
    pub max_complexity: usize,
    pub complex_files: Vec<(PathBuf, usize)>,
    pub avg_function_length: f64,
    pub max_function_length: usize,
    pub long_functions: Vec<(PathBuf, String, usize)>,
}

#[derive(Debug)]
pub struct FileAgeStats {
    pub newest_files: Vec<(PathBuf, String)>,
    pub oldest_files: Vec<(PathBuf, String)>,
    pub most_modified_files: Vec<(PathBuf, usize)>,
}

#[derive(Debug)]
pub struct DuplicateCode {
    pub files: Vec<PathBuf>,
    pub line_count: usize,
    pub similarity: f64,
}

pub fn analyze_repository(repo_path: &Path, history_depth: usize) -> Result<RepositoryAnalysis> {
    println!("Starting repository analysis...");
    println!("Repository path: {}", repo_path.display());

    // Create analysis structure
    let mut analysis = RepositoryAnalysis {
        repo_path: repo_path.to_path_buf(),
        file_count: 0,
        language_stats: HashMap::new(),
        total_lines: 0,
        code_lines: 0,
        comment_lines: 0,
        blank_lines: 0,
        commit_count: 0,
        contributors: Vec::new(),
        last_activity: String::new(),
        file_extensions: HashMap::new(),
        avg_file_size: 0.0,
        largest_files: Vec::new(),
        complexity_stats: ComplexityStats {
            avg_complexity: 0.0,
            max_complexity: 0,
            complex_files: Vec::new(),
            avg_function_length: 0.0,
            max_function_length: 0,
            long_functions: Vec::new(),
        },
        file_age_stats: FileAgeStats {
            newest_files: Vec::new(),
            oldest_files: Vec::new(),
            most_modified_files: Vec::new(),
        },
        duplicate_code: Vec::new(),
        most_changed_files: Vec::new(),
    };

    // Analyze files
    analyze_files(repo_path, &mut analysis)?;

    // Analyze git history
    analyze_git_history(repo_path, &mut analysis, history_depth)?;

    // Analyze code complexity
    analyze_code_complexity(repo_path, &mut analysis)?;

    // Find duplicate code
    find_duplicate_code(repo_path, &mut analysis)?;

    println!("Analysis complete!");
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
        Regex::new(r"dist/").unwrap(),
        Regex::new(r"build/").unwrap(),
        Regex::new(r"\.cache/").unwrap(),
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
                    "xml" => "XML",
                    "gradle" => "Gradle",
                    "tf" | "tfvars" => "Terraform",
                    "proto" => "Protocol Buffers",
                    "graphql" | "gql" => "GraphQL",
                    "r" => "R",
                    "lua" => "Lua",
                    "pl" | "pm" => "Perl",
                    "cs" => "C#",
                    "vb" => "Visual Basic",
                    "scala" => "Scala",
                    "groovy" => "Groovy",
                    "m" => "Objective-C",
                    "mm" => "Objective-C++",
                    _ => "Other",
                };

                *analysis
                    .language_stats
                    .entry(language.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Count lines and analyze code
        if let Ok(content) = std::fs::read_to_string(entry.path()) {
            let (total, code, comment, blank) = count_line_types(&content, entry.path());
            analysis.total_lines += total;
            analysis.code_lines += code;
            analysis.comment_lines += comment;
            analysis.blank_lines += blank;
        }
    }

    Ok(())
}

fn count_line_types(content: &str, path: &Path) -> (usize, usize, usize, usize) {
    let mut total_lines = 0;
    let mut code_lines = 0;
    let mut comment_lines = 0;
    let mut blank_lines = 0;

    let is_comment = |line: &str, in_block_comment: &mut bool| {
        if let Some(ext) = path.extension() {
            match ext.to_str().unwrap_or("").to_lowercase().as_str() {
                "rs" => {
                    // Rust comments
                    if line.trim().starts_with("//") {
                        return true;
                    }
                    if line.trim().starts_with("/*") && !line.trim().contains("*/") {
                        *in_block_comment = true;
                        return true;
                    }
                    if *in_block_comment {
                        if line.trim().contains("*/") {
                            *in_block_comment = false;
                        }
                        return true;
                    }
                }
                "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "cs" | "go" | "swift"
                | "kt" => {
                    // C-style comments
                    if line.trim().starts_with("//") {
                        return true;
                    }
                    if line.trim().starts_with("/*") && !line.trim().contains("*/") {
                        *in_block_comment = true;
                        return true;
                    }
                    if *in_block_comment {
                        if line.trim().contains("*/") {
                            *in_block_comment = false;
                        }
                        return true;
                    }
                }
                "py" | "rb" | "sh" => {
                    // Python/Ruby/Shell comments
                    if line.trim().starts_with("#") {
                        return true;
                    }
                }
                "html" | "xml" => {
                    // HTML/XML comments
                    if line.trim().starts_with("<!--") && !line.trim().contains("-->") {
                        *in_block_comment = true;
                        return true;
                    }
                    if *in_block_comment {
                        if line.trim().contains("-->") {
                            *in_block_comment = false;
                        }
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    };

    let mut in_block_comment = false;

    for line in content.lines() {
        total_lines += 1;

        if line.trim().is_empty() {
            blank_lines += 1;
        } else if is_comment(line, &mut in_block_comment) {
            comment_lines += 1;
        } else {
            code_lines += 1;
        }
    }

    (total_lines, code_lines, comment_lines, blank_lines)
}

fn analyze_git_history(
    repo_path: &Path,
    analysis: &mut RepositoryAnalysis,
    history_depth: usize,
) -> Result<()> {
    println!("Analyzing git history...");

    let (commit_count, contributors, last_activity, file_stats) =
        git::analyze_git_repo_extended(repo_path, history_depth)
            .context("Failed to analyze git repository")?;

    analysis.commit_count = commit_count;
    analysis.contributors = contributors;
    analysis.last_activity = last_activity;

    // Process file age stats
    let mut newest_files: Vec<(PathBuf, String)> = file_stats
        .iter()
        .map(|(path, stats)| (path.clone(), stats.first_commit_date.clone()))
        .collect();
    newest_files.sort_by(|(_, a), (_, b)| b.cmp(a));
    analysis.file_age_stats.newest_files = newest_files.into_iter().take(10).collect();

    let mut oldest_files: Vec<(PathBuf, String)> = file_stats
        .iter()
        .map(|(path, stats)| (path.clone(), stats.first_commit_date.clone()))
        .collect();
    oldest_files.sort_by(|(_, a), (_, b)| a.cmp(b));
    analysis.file_age_stats.oldest_files = oldest_files.into_iter().take(10).collect();

    let mut most_modified_files: Vec<(PathBuf, usize)> = file_stats
        .iter()
        .map(|(path, stats)| (path.clone(), stats.commit_count))
        .collect();
    most_modified_files.sort_by(|(_, a), (_, b)| b.cmp(a));
    analysis.file_age_stats.most_modified_files =
        most_modified_files.into_iter().take(10).collect();

    // Create most changed files info for the report
    let mut most_changed_files = Vec::new();
    for (path, stats) in file_stats.iter() {
        // Find top contributor for this file
        let mut top_contributor = String::from("Unknown");
        let mut max_commits = 0;

        for (author, commit_count) in &stats.author_contributions {
            if *commit_count > max_commits {
                max_commits = *commit_count;
                top_contributor = author.clone();
            }
        }

        most_changed_files.push((
            path.clone(),
            stats.commit_count,
            stats.lines_added,
            stats.lines_removed,
            stats.change_frequency,
            top_contributor,
            stats.last_commit_date.clone(),
            stats.avg_changes_per_commit,
        ));
    }

    // Sort by change frequency
    most_changed_files.sort_by(|(_, _, _, _, a, _, _, _), (_, _, _, _, b, _, _, _)| {
        b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal)
    });

    // Store the top 10 most changed files
    analysis.most_changed_files = most_changed_files.into_iter().take(10).collect();

    Ok(())
}

fn analyze_code_complexity(repo_path: &Path, analysis: &mut RepositoryAnalysis) -> Result<()> {
    println!("Analyzing code complexity...");

    let mut total_complexity = 0;
    let mut file_count = 0;
    let mut complex_files = Vec::new();

    let mut total_function_length = 0;
    let mut function_count = 0;
    let mut long_functions = Vec::new();

    // Patterns to identify functions in different languages
    let function_patterns = HashMap::from([
        ("rs", (Regex::new(r"fn\s+(\w+)\s*\(").unwrap(), Regex::new(r"\{").unwrap(), Regex::new(r"\}").unwrap())),
        ("js", (Regex::new(r"function\s+(\w+)\s*\(|(\w+)\s*=\s*function\s*\(|(\w+)\s*:\s*function\s*\(|(\w+)\s*\([^)]*\)\s*\{").unwrap(), Regex::new(r"\{").unwrap(), Regex::new(r"\}").unwrap())),
        ("ts", (Regex::new(r"function\s+(\w+)\s*\(|(\w+)\s*=\s*function\s*\(|(\w+)\s*:\s*function\s*\(|(\w+)\s*\([^)]*\)\s*\{").unwrap(), Regex::new(r"\{").unwrap(), Regex::new(r"\}").unwrap())),
        ("py", (Regex::new(r"def\s+(\w+)\s*\(").unwrap(), Regex::new(r":").unwrap(), Regex::new(r"^\s*$|^\s*\w").unwrap())),
        ("java", (Regex::new(r"(public|private|protected|static|\s) +[\w<>\[\]]+\s+(\w+) *\([^)]*\) *\{?").unwrap(), Regex::new(r"\{").unwrap(), Regex::new(r"\}").unwrap())),
        ("go", (Regex::new(r"func\s+(\w+)\s*\(").unwrap(), Regex::new(r"\{").unwrap(), Regex::new(r"\}").unwrap())),
    ]);

    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path(), &ignore_patterns()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_str().unwrap_or("").to_lowercase();

            if let Some((func_pattern, open_pattern, _close_pattern)) =
                function_patterns.get(ext_str.as_str())
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let complexity = calculate_cyclomatic_complexity(&content, &ext_str);
                    total_complexity += complexity;
                    file_count += 1;

                    if complexity > 10 {
                        complex_files.push((entry.path().to_path_buf(), complexity));
                    }

                    // Analyze function lengths
                    let functions = find_functions(
                        &content,
                        func_pattern,
                        open_pattern,
                        _close_pattern,
                        &ext_str,
                    );
                    for (name, length) in functions {
                        total_function_length += length;
                        function_count += 1;

                        if length > 30 {
                            long_functions.push((entry.path().to_path_buf(), name, length));
                        }
                    }
                }
            }
        }
    }

    // Calculate averages
    if file_count > 0 {
        analysis.complexity_stats.avg_complexity = total_complexity as f64 / file_count as f64;
    }

    if function_count > 0 {
        analysis.complexity_stats.avg_function_length =
            total_function_length as f64 / function_count as f64;
    }

    // Sort and store results
    complex_files.sort_by(|(_, a), (_, b)| b.cmp(a));
    analysis.complexity_stats.complex_files = complex_files.into_iter().take(10).collect();

    if let Some((_, complexity)) = analysis.complexity_stats.complex_files.first() {
        analysis.complexity_stats.max_complexity = *complexity;
    }

    long_functions.sort_by(|(_, _, a), (_, _, b)| b.cmp(a));
    analysis.complexity_stats.long_functions = long_functions.into_iter().take(10).collect();

    if let Some((_, _, length)) = analysis.complexity_stats.long_functions.first() {
        analysis.complexity_stats.max_function_length = *length;
    }

    Ok(())
}

fn calculate_cyclomatic_complexity(content: &str, ext: &str) -> usize {
    // Base complexity is 1
    let mut complexity = 1;

    match ext {
        "rs" | "js" | "ts" | "java" | "c" | "cpp" | "cs" | "go" | "swift" | "kt" | "scala" => {
            // Count control flow structures
            for line in content.lines() {
                let line = line.trim();

                // Skip comments
                if line.starts_with("//") || line.starts_with("/*") || line.starts_with("*") {
                    continue;
                }

                // Count conditional statements
                if line.contains("if ")
                    || line.contains("else if")
                    || line.contains(" ? ")  // Ternary operator
                    || line.contains("for ")
                    || line.contains("while ")
                    || line.contains("case ")
                    || line.contains("catch ")
                    || line.contains("switch ")
                    || (ext == "rs" && line.contains("match "))
                    || (ext == "go" && line.contains("select "))
                    || (ext == "swift" && line.contains("guard "))
                {
                    complexity += 1;
                }

                // Count logical operators (each represents a branch)
                complexity += line.matches("&&").count();
                complexity += line.matches("||").count();
            }
        }
        "py" => {
            // Count control flow structures for Python
            for line in content.lines() {
                let line = line.trim();

                // Skip comments
                if line.starts_with("#") {
                    continue;
                }

                if line.contains("if ")
                    || line.contains("elif ")
                    || line.contains("for ")
                    || line.contains("while ")
                    || line.contains("except ")
                    || line.contains("with ")
                    || line.contains("comprehension")
                {
                    complexity += 1;
                }

                // Count logical operators
                complexity += line.matches(" and ").count();
                complexity += line.matches(" or ").count();
            }
        }
        "rb" => {
            // Ruby
            for line in content.lines() {
                let line = line.trim();

                // Skip comments
                if line.starts_with("#") {
                    continue;
                }

                if line.contains("if ")
                    || line.contains("elsif ")
                    || line.contains("unless ")
                    || line.contains("case ")
                    || line.contains("when ")
                    || line.contains("for ")
                    || line.contains("while ")
                    || line.contains("until ")
                    || line.contains("rescue ")
                {
                    complexity += 1;
                }

                // Count logical operators
                complexity += line.matches("&&").count();
                complexity += line.matches("||").count();
            }
        }
        "php" => {
            // PHP
            for line in content.lines() {
                let line = line.trim();

                // Skip comments
                if line.starts_with("//") || line.starts_with("/*") || line.starts_with("*") {
                    continue;
                }

                if line.contains("if ")
                    || line.contains("elseif ")
                    || line.contains("for ")
                    || line.contains("foreach ")
                    || line.contains("while ")
                    || line.contains("case ")
                    || line.contains("catch ")
                {
                    complexity += 1;
                }

                // Count logical operators
                complexity += line.matches("&&").count();
                complexity += line.matches("||").count();
                complexity += line.matches(" and ").count();
                complexity += line.matches(" or ").count();
            }
        }
        _ => {}
    }

    complexity
}

fn find_functions(
    content: &str,
    func_pattern: &Regex,
    open_pattern: &Regex,
    _close_pattern: &Regex,
    ext: &str,
) -> Vec<(String, usize)> {
    let mut functions = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut i = 0;
    while i < lines.len() {
        if let Some(captures) = func_pattern.captures(lines[i]) {
            let mut func_name = String::new();
            for j in 1..captures.len() {
                if let Some(m) = captures.get(j) {
                    if !m.as_str().is_empty() {
                        func_name = m.as_str().to_string();
                        break;
                    }
                }
            }

            if func_name.is_empty() {
                func_name = "anonymous".to_string();
            }

            // Find function body
            let mut start_line = i;

            // Find opening brace
            while start_line < lines.len() && !open_pattern.is_match(lines[start_line]) {
                start_line += 1;
            }

            let mut end_line;
            if ext == "py" {
                // Python functions are indentation-based
                let base_indent = lines[start_line]
                    .chars()
                    .take_while(|c| c.is_whitespace())
                    .count();
                end_line = start_line + 1;

                while end_line < lines.len() {
                    let indent = lines[end_line]
                        .chars()
                        .take_while(|c| c.is_whitespace())
                        .count();
                    if !lines[end_line].trim().is_empty() && indent <= base_indent {
                        break;
                    }
                    end_line += 1;
                }
            } else {
                // Brace-based languages
                let mut brace_count = 1;
                end_line = start_line + 1;

                while end_line < lines.len() && brace_count > 0 {
                    if lines[end_line].contains('{') {
                        brace_count += lines[end_line].matches('{').count();
                    }
                    if lines[end_line].contains('}') {
                        brace_count -= lines[end_line].matches('}').count();
                    }
                    if brace_count == 0 {
                        break;
                    }
                    end_line += 1;
                }
            }

            let function_length = end_line - start_line;
            functions.push((func_name, function_length));

            i = end_line;
        } else {
            i += 1;
        }
    }

    functions
}

fn find_duplicate_code(repo_path: &Path, analysis: &mut RepositoryAnalysis) -> Result<()> {
    println!("Finding duplicate code...");

    // Simple duplicate code detection using line hashing
    let mut file_contents: HashMap<PathBuf, Vec<String>> = HashMap::new();

    // Read file contents
    for entry in WalkDir::new(repo_path)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path(), &ignore_patterns()))
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        if let Some(ext) = entry.path().extension() {
            let ext_str = ext.to_str().unwrap_or("").to_lowercase();

            // Only analyze source code files
            if ["rs", "js", "ts", "py", "java", "c", "cpp", "go", "cs"].contains(&ext_str.as_str())
            {
                if let Ok(content) = std::fs::read_to_string(entry.path()) {
                    let lines: Vec<String> = content
                        .lines()
                        .map(|l| l.trim().to_string())
                        .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#"))
                        .collect();

                    file_contents.insert(entry.path().to_path_buf(), lines);
                }
            }
        }
    }

    // Find duplicate blocks (simple approach)
    let min_block_size = 6; // Minimum number of lines to consider a duplicate
    let mut duplicates = Vec::new();

    let files: Vec<PathBuf> = file_contents.keys().cloned().collect();

    for i in 0..files.len() {
        for j in (i + 1)..files.len() {
            let file1 = &files[i];
            let file2 = &files[j];

            let lines1 = file_contents.get(file1).unwrap();
            let lines2 = file_contents.get(file2).unwrap();

            let mut duplicate_blocks = Vec::new();

            for start1 in 0..(lines1.len().saturating_sub(min_block_size)) {
                'outer: for start2 in 0..(lines2.len().saturating_sub(min_block_size)) {
                    let mut block_size = 0;

                    while start1 + block_size < lines1.len()
                        && start2 + block_size < lines2.len()
                        && lines1[start1 + block_size] == lines2[start2 + block_size]
                    {
                        block_size += 1;
                    }

                    if block_size >= min_block_size {
                        // Check if this block overlaps with any existing block
                        for (s1, s2, size) in &duplicate_blocks {
                            if (start1 >= *s1 && start1 < s1 + size)
                                || (start2 >= *s2 && start2 < s2 + size)
                            {
                                continue 'outer;
                            }
                        }

                        duplicate_blocks.push((start1, start2, block_size));
                    }
                }
            }

            for (_, _, size) in duplicate_blocks {
                if size >= min_block_size {
                    let mut files_vec = Vec::new();
                    files_vec.push(file1.clone());
                    files_vec.push(file2.clone());

                    duplicates.push(DuplicateCode {
                        files: files_vec,
                        line_count: size,
                        similarity: 1.0, // Perfect match
                    });
                }
            }
        }
    }

    // Sort by line count and take top 10
    duplicates.sort_by(|a, b| b.line_count.cmp(&a.line_count));
    analysis.duplicate_code = duplicates.into_iter().take(10).collect();

    Ok(())
}

fn ignore_patterns() -> Vec<Regex> {
    vec![
        Regex::new(r"\.git/").unwrap(),
        Regex::new(r"node_modules/").unwrap(),
        Regex::new(r"target/").unwrap(),
        Regex::new(r"\.DS_Store").unwrap(),
        Regex::new(r"\.idea/").unwrap(),
        Regex::new(r"\.vscode/").unwrap(),
        Regex::new(r"dist/").unwrap(),
        Regex::new(r"build/").unwrap(),
        Regex::new(r"\.cache/").unwrap(),
    ]
}

fn is_ignored(path: &Path, patterns: &[Regex]) -> bool {
    let path_str = path.to_string_lossy();
    patterns.iter().any(|pattern| pattern.is_match(&path_str))
}
