// Export modules
pub mod analyzer;
pub mod cli;
pub mod config;
pub mod git;
pub mod report;
pub mod s3;

// Re-export main types for convenience
pub use analyzer::RepositoryAnalysis;
pub use cli::Cli;
pub use config::Config;

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
