use anyhow::Result;
use aws_sdk_s3::{primitives::ByteStream, Client};
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

    // Set AWS credentials as environment variables from config
    let config = crate::config::Config::load()?;
    std::env::set_var("AWS_ACCESS_KEY_ID", &config.aws.access_key);
    std::env::set_var("AWS_SECRET_ACCESS_KEY", &config.aws.secret_key);
    std::env::set_var("AWS_REGION", region);

    println!("AWS credentials set from config");

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
