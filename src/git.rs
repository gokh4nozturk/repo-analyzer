use anyhow::{Context, Result};
use chrono::{DateTime, Local, TimeZone};
use git2::{Repository, Time};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Contributor {
    pub name: String,
    pub email: String,
    pub commit_count: usize,
    pub first_commit: String,
    pub last_commit: String,
}

pub fn analyze_git_repo(repo_path: &Path) -> Result<(usize, Vec<Contributor>, String)> {
    let repo = Repository::open(repo_path).context("Failed to open git repository")?;

    let mut commit_count = 0;
    let mut contributors_map: HashMap<String, Contributor> = HashMap::new();
    let mut last_commit_time = None;

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

    for oid in revwalk {
        let oid = oid.context("Failed to get commit OID")?;
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

    Ok((commit_count, contributors, last_activity))
}

fn format_git_time(time: &Time) -> String {
    let dt: DateTime<Local> = Local.timestamp_opt(time.seconds(), 0).unwrap();
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}
