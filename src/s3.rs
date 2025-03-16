use anyhow::Result;
use aws_sdk_s3::{primitives::ByteStream, Client};
use std::fs;
use std::path::Path;

/// Uploads a file to an S3 bucket and returns the public URL
pub async fn upload_to_s3(
    file_path: &Path,
    bucket_name: &str,
    key: &str,
    region: &str,
) -> Result<String> {
    println!("Starting S3 upload process...");
    println!("File: {}", file_path.display());
    println!("Bucket: {}", bucket_name);
    println!("Key: {}", key);
    println!("Region: {}", region);

    // First try direct S3 upload with local credentials
    match direct_s3_upload(file_path, bucket_name, key, region).await {
        Ok(url) => {
            println!("Direct S3 upload successful");
            return Ok(url);
        }
        Err(e) => {
            println!("Direct S3 upload failed: {}", e);
            println!("Trying upload via central API...");
            // Fall back to API-based upload
            return upload_via_api(file_path, bucket_name, key, region).await;
        }
    }
}

/// Attempts to upload directly to S3 using local credentials
async fn direct_s3_upload(
    file_path: &Path,
    bucket_name: &str,
    key: &str,
    region: &str,
) -> Result<String> {
    // Check if AWS credentials are already set in environment variables
    let access_key_set = std::env::var("AWS_ACCESS_KEY_ID").is_ok();
    let secret_key_set = std::env::var("AWS_SECRET_ACCESS_KEY").is_ok();

    // Only set from config if not already set in environment
    if !access_key_set || !secret_key_set {
        // Set AWS credentials as environment variables from config
        let config = crate::config::Config::load()?;

        if !access_key_set {
            std::env::set_var("AWS_ACCESS_KEY_ID", &config.aws.access_key);
        }

        if !secret_key_set {
            std::env::set_var("AWS_SECRET_ACCESS_KEY", &config.aws.secret_key);
        }

        println!("AWS credentials set from config");
    } else {
        println!("Using AWS credentials from environment variables");
    }

    // Always set region from parameter
    std::env::set_var("AWS_REGION", region);

    // Load AWS configuration
    let aws_config = aws_config::from_env()
        .region(aws_types::region::Region::new(region.to_string()))
        .load()
        .await;

    // Create S3 client
    let client = Client::new(&aws_config);
    println!("AWS SDK client created");

    // Read file content
    println!("Reading file content...");
    let body = match ByteStream::from_path(file_path).await {
        Ok(stream) => {
            println!("File read successfully");
            stream
        }
        Err(e) => {
            println!("Error reading file: {:?}", e);
            return Err(anyhow::anyhow!("Failed to read file: {:?}", e));
        }
    };

    // Upload file to S3
    println!("Uploading to S3...");
    match client
        .put_object()
        .bucket(bucket_name)
        .key(key)
        .body(body)
        .acl(aws_sdk_s3::types::ObjectCannedAcl::PublicRead)
        .send()
        .await
    {
        Ok(_) => {
            println!("Upload successful");
        }
        Err(e) => {
            println!("S3 upload error: {:?}", e);
            return Err(anyhow::anyhow!("Failed to upload file to S3: {:?}", e));
        }
    };

    // Generate public URL
    let url = format!(
        "https://{}.s3.{}.amazonaws.com/{}",
        bucket_name, region, key
    );

    println!("Generated URL: {}", url);
    Ok(url)
}

/// Uploads the file via a central API service that handles S3 uploads with proper credentials
async fn upload_via_api(
    file_path: &Path,
    bucket_name: &str,
    key: &str,
    region: &str,
) -> Result<String> {
    // Get API URL from environment variable or config
    let api_url = std::env::var("REPO_ANALYZER_API_URL").unwrap_or_else(|_| {
        // Try to get from config if environment variable is not set
        match crate::config::Config::load() {
            Ok(config) => config
                .api_url
                .unwrap_or_else(|| "https://api.repo-analyzer.com/upload".to_string()),
            Err(_) => "https://api.repo-analyzer.com/upload".to_string(),
        }
    });

    println!("Uploading via central API: {}", api_url);

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

    // Create a multipart form with the file and metadata
    let form = reqwest::multipart::Form::new()
        .text("bucket", bucket_name.to_string())
        .text("key", key.to_string())
        .text("region", region.to_string())
        .part(
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

        println!("API upload successful");
        println!("Generated URL: {}", url);
        Ok(url)
    } else {
        let error_text = response.text().await?;
        Err(anyhow::anyhow!("API upload failed: {}", error_text))
    }
}
