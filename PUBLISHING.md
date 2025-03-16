# Crates.io Publishing

This document describes the steps to publish the repo-analyzer project to crates.io.

## Automatic Publishing with GitHub Actions

This project has a configuration for automatic publishing to crates.io using GitHub Actions. To publish a new version:

1. Update the `version` in the `Cargo.toml` file (e.g. "0.1.0" -> "0.1.1")
2. Commit the changes
3. Create a new git tag with the new version and push it:
   ```bash
   git tag v0.1.1
   git push origin v0.1.1
   ```

GitHub Actions will automatically trigger and publish the new version to crates.io.

## GitHub Secrets

To allow GitHub Actions to publish to crates.io, you need to create a secret in your GitHub repository:

1. Go to the repo page on GitHub
2. Click on "Settings" > "Secrets and variables" > "Actions"
3. Click on "New repository secret"
4. Name it `CARGO_REGISTRY_TOKEN`
5. Set the value to your crates.io API token
6. Click "Add secret"

## Manual Publishing

If you don't want to use GitHub Actions, you can manually publish by following these steps:

### Preparations

1. Update the `authors`, `repository` and other metadata in the `Cargo.toml` file with your own information:
   - `authors` to your name and email address
   - `repository` to your GitHub repo URL
   - Update other metadata as needed

2. Update the copyright information in the `LICENSE` file with your own information.

3. Replace all "yourusername" references in the `README.md` file with your GitHub username.

### crates.io Account

1. If you don't have a crates.io account yet, register at https://crates.io.

2. After creating your account, create a new API token:
   - Log in to crates.io
   - Click on your username in the top right corner
   - Select "Account Settings"
   - Create a new token in the "API Tokens" section

3. Save your API token to Cargo:
   ```bash
   cargo login YOUR_API_TOKEN
   ```

### Preparing and Publishing the Package

1. Test your package:
   ```bash
   cargo package
   ```

2. Publish your package:
   ```bash
   cargo publish
   ```

### After Publishing

1. Check if your package is visible on crates.io:
   ```
   https://crates.io/crates/repo-analyzer
   ```

2. Try installing your package with cargo:
   ```bash
   cargo install repo-analyzer
   ```

## Important Notes

- Once a package is published to crates.io, you cannot delete it. You can publish new versions, but old versions will always be accessible.
- Package names must be unique. If "repo-analyzer" is already taken, you will need to choose a different name.
- Your package's dependencies must also be published to crates.io.