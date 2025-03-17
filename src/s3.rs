use anyhow::Result;
use std::fs;
use std::path::Path;

/// Uploads a file to cloud storage and returns the public URL
pub async fn upload_to_s3(
    file_path: &Path,
    bucket_name: &str,
    key: &str,
    region: &str,
    use_api: bool,
) -> Result<String> {
    println!("Starting upload process...");
    println!("File: {}", file_path.display());

    // Always use the API for simplicity
    upload_via_api(file_path).await
}

/// Uploads the file via the API service
async fn upload_via_api(file_path: &Path) -> Result<String> {
    // API URL
    let api_url = std::env::var("REPO_ANALYZER_API_URL").unwrap_or_else(|_| {
        // Try to get from config if environment variable is not set
        match crate::config::Config::load() {
            Ok(config) => config
                .api_url
                .unwrap_or_else(|| "https://api.analyzer.gokhanozturk.io/api/upload".to_string()),
            Err(_) => "https://api.analyzer.gokhanozturk.io/api/upload".to_string(),
        }
    });

    println!("Uploading via API: {}", api_url);

    // Read file content
    let file_content = fs::read(file_path)?;

    // Get API key from environment or config
    let api_key = std::env::var("REPO_ANALYZER_API_KEY").unwrap_or_else(|_| {
        // Try to get from config if environment variable is not set
        match crate::config::Config::load() {
            Ok(config) => config.api_key.unwrap_or_else(|| "".to_string()),
            Err(_) => "".to_string(),
        }
    });

    // Create a multipart form with the file
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(file_content)
            .file_name(file_path.file_name().unwrap().to_string_lossy().to_string()),
    );

    // Send the request to the API
    let client = reqwest::Client::new();
    let mut request = client.post(api_url).multipart(form);

    // Add API key header if available
    if !api_key.is_empty() {
        request = request.header("x-api-key", api_key);
    }

    let response = request.send().await?;

    // Check if the request was successful
    if response.status().is_success() {
        // Parse the response to get the URL
        let response_json: serde_json::Value = response.json().await?;
        let url = response_json["url"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid response from API: missing URL"))?
            .to_string();

        println!("Upload successful");
        println!("Generated URL: {}", url);
        Ok(url)
    } else {
        let error_text = response.text().await?;
        Err(anyhow::anyhow!("Upload failed: {}", error_text))
    }
}
