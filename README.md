# Repository Analyzer

A command-line tool written in Rust to analyze GitHub repositories and generate comprehensive reports with automatic cloud storage upload capability.

## Features

- Analyze file structure and language distribution
- Count lines of code
- Track commit history and contributor statistics
- Generate reports in multiple formats (text, JSON, HTML)
- Identify file extensions and their distribution
- Automatically upload reports to cloud storage (S3 or R2) for easy sharing and access
- Analyze code complexity and identify potential issues

## Installation

### From crates.io

```bash
cargo install repo-analyzer
```

### Prerequisites

- Rust and Cargo (1.70.0 or newer)
- Git (for repository analysis)
- AWS account with S3 access (for direct S3 upload) or use our API service (no credentials needed)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/gokh4nozturk/repo-analyzer.git
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
    "region": "eu-central-1",
    "bucket": "repo-analyzer"
  }
}
```

Replace the placeholder values with your actual AWS credentials and ensure the bucket exists in the specified region.

## S3 Upload Feature

The tool can optionally upload generated reports to an S3 bucket for easy sharing and access. This feature is disabled by default and can be enabled with the `--upload-to-s3` flag.

### How Report Uploads Work

The tool uses a two-step approach for uploading reports:

1. **Direct S3 Upload**: First, it attempts to upload directly to S3 using locally configured AWS credentials.

2. **Central API Fallback**: If direct upload fails (e.g., due to missing or invalid credentials), the tool automatically falls back to uploading via our central API service, which securely handles the S3 upload process.

This approach ensures that reports can be uploaded to our central repository even if you don't have AWS credentials configured locally.

### API Service

The repository includes an API service in the `api/` directory that handles S3 uploads. This service can be deployed to any Node.js hosting platform and used as the central upload point for all repo-analyzer clients.

To set up the API service:

1. Navigate to the `api/` directory
2. Copy `.env.example` to `.env` and configure your AWS credentials
3. Install dependencies: `npm install`
4. Start the service: `npm start`

For more details, see the [API README](api/README.md).

### Configuring AWS Credentials (Optional)

If you want to use your own S3 bucket, you can configure AWS credentials in one of the following ways:

1. **Environment Variables**:
   ```
   export AWS_ACCESS_KEY_ID=your_access_key
   export AWS_SECRET_ACCESS_KEY=your_secret_key
   export AWS_REGION=your_region
   export AWS_S3_BUCKET=your_bucket_name
   ```

2. **Config File**:
   Create a `config.json` file in the same directory as the executable with the following structure:
   ```json
   {
     "aws": {
       "access_key": "your_access_key",
       "secret_key": "your_secret_key",
       "region": "your_region",
       "bucket": "your_bucket_name"
     }
   }
   ```

If you don't configure AWS credentials, the tool will automatically use our central API service to upload reports to our shared repository.

## Usage

```bash
# Basic usage
repo-analyzer --repo-path /path/to/repository

# Analyze current directory
repo-analyzer --repo-path .

# Specify output format (default is HTML)
repo-analyzer --repo-path /path/to/repository --output-format json

# Show more contributors
repo-analyzer --repo-path /path/to/repository --top-contributors 10

# Analyze a remote repository
repo-analyzer --remote-url https://github.com/username/repository
```

### Command-line Options

- `--repo-path, -r`: Path to the repository to analyze (required unless --remote-url is provided)
- `--remote-url, -u`: URL of a remote repository to clone and analyze
- `--output-format, -o`: Output format (text, json, html) (default: html)
- `--top-contributors, -t`: Number of top contributors to show (default: 5)
- `--history-depth`: Depth of commit history to analyze (0 for all) (default: 0)

## Report Access

After analyzing a repository, the tool automatically uploads the report to your S3 bucket and provides a URL where you can access the report. This URL is displayed in the console output after the analysis is complete.

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

## Publishing

This project uses GitHub Actions for automated publishing to crates.io. To publish a new version:

1. Update the version number in `Cargo.toml`
2. Commit your changes
3. Create and push a new tag with the version number:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

The GitHub Actions workflow will automatically build, test, and publish the new version to crates.io. 