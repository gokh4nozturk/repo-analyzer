use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{build::RepoBuilder, FetchOptions, RemoteCallbacks, Repository, Time};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Contributor {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
    pub first_commit: String,
    pub last_commit: String,
}

#[derive(Debug, Clone)]
pub struct FileStats {
    pub commit_count: usize,
    pub first_commit_date: String,
    pub last_commit_date: String,
    pub authors: Vec<String>,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub change_frequency: f64,                        // Changes per month
    pub author_contributions: HashMap<String, usize>, // Author -> commit count
    pub last_modified_by: String,
    pub avg_changes_per_commit: f64,
}

pub fn clone_repository(url: &str, target_path: &Path) -> Result<Repository> {
    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!(
                "\rReceiving objects: 100% ({}/{}), {:.2} KiB\r",
                stats.received_objects(),
                stats.total_objects(),
                stats.received_bytes() as f64 / 1024.0
            );
        } else if stats.total_objects() > 0 {
            print!(
                "\rReceiving objects: {}% ({}/{}), {:.2} KiB\r",
                (stats.received_objects() * 100) / stats.total_objects(),
                stats.received_objects(),
                stats.total_objects(),
                stats.received_bytes() as f64 / 1024.0
            );
        }
        std::io::stdout().flush().unwrap_or(());
        true
    });

    let mut fetch_options = FetchOptions::new();
    fetch_options.remote_callbacks(callbacks);

    let repo = RepoBuilder::new()
        .fetch_options(fetch_options)
        .clone(url, target_path)
        .context("Failed to clone repository")?;

    println!("\nRepository cloned successfully");
    Ok(repo)
}

pub fn analyze_git_repo(
    repo_path: &Path,
    depth: usize,
) -> Result<(usize, Vec<Contributor>, String)> {
    let (commit_count, contributors, last_activity, _) =
        analyze_git_repo_extended(repo_path, depth)?;
    Ok((commit_count, contributors, last_activity))
}

pub fn analyze_git_repo_extended(
    repo_path: &Path,
    depth: usize,
) -> Result<(usize, Vec<Contributor>, String, HashMap<PathBuf, FileStats>)> {
    let repo = Repository::open(repo_path).context("Failed to open git repository")?;

    let mut commit_count = 0;
    let mut contributors_map: HashMap<String, Contributor> = HashMap::new();
    let mut last_commit_time = None;
    let mut file_stats: HashMap<PathBuf, FileStats> = HashMap::new();

    // Get the HEAD reference
    let head = repo.head().context("Failed to get HEAD reference")?;

    // Get the commit that HEAD points to
    let obj = head
        .peel(git2::ObjectType::Commit)
        .context("Failed to peel to commit")?;
    let commit = obj
        .into_commit()
        .map_err(|_| anyhow::anyhow!("Failed to convert to commit"))?;

    // Create a revwalk to iterate through the commit history
    let mut revwalk = repo.revwalk().context("Failed to create revwalk")?;
    revwalk
        .push(commit.id())
        .context("Failed to push commit to revwalk")?;

    for (i, oid_result) in revwalk.enumerate() {
        // If depth is set and we've reached it, break
        if depth > 0 && i >= depth {
            break;
        }

        let oid = oid_result.context("Failed to get commit OID")?;
        let commit = repo.find_commit(oid).context("Failed to find commit")?;

        commit_count += 1;

        // Get commit author
        let author = commit.author();
        let time = commit.time();
        let datetime = format_git_time(&time);

        // Update last commit time
        if last_commit_time.is_none() || time.seconds() > last_commit_time.unwrap() {
            last_commit_time = Some(time.seconds());
        }

        // Update contributor information
        let key = format!(
            "{} <{}>",
            author.name().unwrap_or("Unknown"),
            author.email().unwrap_or("unknown")
        );

        contributors_map
            .entry(key.clone())
            .and_modify(|contributor| {
                contributor.commit_count += 1;
                contributor.last_commit = datetime.clone();
            })
            .or_insert_with(|| Contributor {
                name: author.name().unwrap_or("Unknown").to_string(),
                email: author.email().unwrap_or("unknown").to_string(),
                commit_count: 1,
                first_commit: datetime.clone(),
                last_commit: datetime.clone(),
            });

        // Get file changes in this commit
        if let Ok(parent) = commit.parent(0) {
            let diff = repo
                .diff_tree_to_tree(
                    Some(&parent.tree().unwrap()),
                    Some(&commit.tree().unwrap()),
                    None,
                )
                .unwrap();

            let mut lines_added_map: HashMap<PathBuf, usize> = HashMap::new();
            let mut lines_removed_map: HashMap<PathBuf, usize> = HashMap::new();
            let mut files_changed: HashSet<PathBuf> = HashSet::new();

            diff.foreach(
                &mut |delta, _| {
                    if let Some(path) = delta.new_file().path() {
                        files_changed.insert(repo_path.join(path));
                    }
                    true
                },
                None,
                Some(&mut |delta, hunk| {
                    if let Some(path) = delta.new_file().path() {
                        let path_buf = repo_path.join(path);
                        *lines_added_map.entry(path_buf.clone()).or_insert(0) +=
                            hunk.new_lines() as usize;
                        *lines_removed_map.entry(path_buf).or_insert(0) +=
                            hunk.old_lines() as usize;
                    }
                    true
                }),
                None,
            )
            .unwrap();

            // Now update file_stats with the collected information
            for path in files_changed {
                let author_name = author.name().unwrap_or("Unknown").to_string();
                let added = lines_added_map.get(&path).cloned().unwrap_or(0);
                let removed = lines_removed_map.get(&path).cloned().unwrap_or(0);

                // Check if we already have stats for this file
                if let Some(stats) = file_stats.get_mut(&path) {
                    // Update existing stats
                    stats.commit_count += 1;
                    stats.last_commit_date = datetime.clone();
                    stats.last_modified_by = author_name.clone();
                    stats.lines_added += added;
                    stats.lines_removed += removed;

                    // Update author contributions
                    *stats
                        .author_contributions
                        .entry(author_name.clone())
                        .or_insert(0) += 1;

                    if !stats.authors.contains(&author_name) {
                        stats.authors.push(author_name);
                    }
                } else {
                    // Create new stats
                    let mut authors = Vec::new();
                    authors.push(author_name.clone());

                    let mut author_contributions = HashMap::new();
                    author_contributions.insert(author_name.clone(), 1);

                    let new_stats = FileStats {
                        commit_count: 1,
                        first_commit_date: datetime.clone(),
                        last_commit_date: datetime.clone(),
                        authors,
                        lines_added: added,
                        lines_removed: removed,
                        change_frequency: 0.0,
                        author_contributions,
                        last_modified_by: author_name,
                        avg_changes_per_commit: 0.0,
                    };

                    file_stats.insert(path, new_stats);
                }
            }
        }
    }

    // Calculate additional statistics for each file
    for stats in file_stats.values_mut() {
        // Calculate change frequency (changes per month)
        if let (Ok(first_date), Ok(last_date)) = (
            chrono::DateTime::parse_from_str(&stats.first_commit_date, "%Y-%m-%d %H:%M:%S %z"),
            chrono::DateTime::parse_from_str(&stats.last_commit_date, "%Y-%m-%d %H:%M:%S %z"),
        ) {
            let duration = last_date.signed_duration_since(first_date);
            let months = (duration.num_days() as f64) / 30.0;

            if months > 0.0 {
                stats.change_frequency = stats.commit_count as f64 / months;
            } else {
                stats.change_frequency = stats.commit_count as f64; // All changes in less than a month
            }
        }

        // Calculate average changes per commit
        let total_changes = stats.lines_added + stats.lines_removed;
        if stats.commit_count > 0 {
            stats.avg_changes_per_commit = total_changes as f64 / stats.commit_count as f64;
        }
    }

    // Sort contributors by commit count
    let mut contributors: Vec<Contributor> = contributors_map.values().cloned().collect();
    contributors.sort_by(|a, b| b.commit_count.cmp(&a.commit_count));

    // Format last activity time
    let last_activity = if let Some(time) = last_commit_time {
        let dt = Local.timestamp_opt(time, 0).unwrap();
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Unknown".to_string()
    };

    Ok((commit_count, contributors, last_activity, file_stats))
}

fn format_git_time(time: &Time) -> String {
    let dt: DateTime<Local> = Local.timestamp_opt(time.seconds(), 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
