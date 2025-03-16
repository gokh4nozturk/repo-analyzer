# Repository Analyzer

A command-line tool written in Rust to analyze GitHub repositories and generate comprehensive reports.

## Features

- Analyze file structure and language distribution
- Count lines of code
- Track commit history and contributor statistics
- Generate reports in multiple formats (text, JSON, HTML)
- Identify file extensions and their distribution
- Automatically upload reports to cloud storage for easy sharing

## Installation

### Prerequisites

- Rust and Cargo (1.70.0 or newer)
- Git (for repository analysis)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/yourusername/repo-analyzer.git
cd repo-analyzer

# Configure AWS credentials
cp config.json.example config.json
# Edit config.json with your AWS credentials

# Build the project
cargo build --release

# The binary will be available at target/release/repo-analyzer
```

### AWS Configuration

The tool requires AWS credentials to upload reports to S3. Create a `config.json` file in the project root with the following structure:

```json
{
  "aws": {
    "access_key": "YOUR_ACCESS_KEY",
    "secret_key": "YOUR_SECRET_KEY",
    "region": "us-east-1",
    "bucket": "repo-analyzer"
  }
}
```

Replace the placeholder values with your actual AWS credentials.

## Usage

```bash
# Basic usage
repo-analyzer --repo-path /path/to/repository

# Specify output format (default is HTML)
repo-analyzer --repo-path /path/to/repository --output-format json

# Show detailed history
repo-analyzer --repo-path /path/to/repository --detailed-history

# Show more contributors
repo-analyzer --repo-path /path/to/repository --top-contributors 10

# Analyze a remote repository
repo-analyzer --remote-url https://github.com/username/repository
```

### Command-line Options

- `--repo-path, -r`: Path to the repository to analyze (required unless --remote-url is provided)
- `--remote-url, -u`: URL of a remote repository to clone and analyze
- `--output-format, -o`: Output format (text, json, html) (default: html)
- `--detailed-history, -d`: Include detailed commit history (default: false)
- `--top-contributors, -t`: Number of top contributors to show (default: 5)
- `--history-depth`: Depth of commit history to analyze (0 for all) (default: 0)

## Report Access

After analyzing a repository, the tool automatically uploads the report to cloud storage and provides a URL where you can access the report. This URL is displayed in the console output after the analysis is complete.

The report URL is permanent and can be shared with others to provide access to the repository analysis.

## Example Output

### Text Output

```
Repository Analysis Report
=========================

General Information:
Repository Path: /path/to/repository
Total Files: 120
Total Lines of Code: 15000
Total Commits: 250
Last Activity: 2023-03-15 14:30:45

Language Statistics:
Rust: 45 files (37.5%)
JavaScript: 30 files (25.0%)
HTML: 15 files (12.5%)
CSS: 10 files (8.3%)
Other: 20 files (16.7%)

File Extensions:
.rs: 45 files (37.5%)
.js: 30 files (25.0%)
.html: 15 files (12.5%)
.css: 10 files (8.3%)
.json: 8 files (6.7%)
.md: 7 files (5.8%)
.toml: 5 files (4.2%)

Top Contributors:
1. John Doe <john@example.com> - 120 commits (first: 2022-01-15 10:30:00, last: 2023-03-15 14:30:45)
2. Jane Smith <jane@example.com> - 80 commits (first: 2022-02-10 09:15:30, last: 2023-03-10 11:45:20)
3. Bob Johnson <bob@example.com> - 50 commits (first: 2022-03-05 14:20:10, last: 2023-02-28 16:30:15)
```

## License

MIT

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 