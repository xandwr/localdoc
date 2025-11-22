use anyhow::{Context, Result};
use colored::Colorize;
use std::path::PathBuf;
use std::process::Command;

/// Generate a docpack from a source file or GitHub URL
pub fn run(input: String) -> Result<()> {
    let zip_path = if is_github_url(&input) {
        println!("\n{}", "Detected GitHub URL".bright_cyan().bold());
        download_github_repo(&input)?
    } else {
        let path = PathBuf::from(&input);
        // Verify input exists
        if !path.exists() {
            anyhow::bail!("Input file does not exist: {:?}", path);
        }

        // Verify it's a zip file
        if path.extension().and_then(|s| s.to_str()) != Some("zip") {
            anyhow::bail!("Input must be a .zip file, got: {:?}", path);
        }
        path
    };

    println!("\n{}", "Generating Docpack".bright_cyan().bold());
    println!("{}", format!("Input: {:?}", input).bright_black());
    println!("{}", "=".repeat(80).bright_black());

    // Find the builder binary
    let builder_path = find_builder_binary()?;
    println!(
        "\n{}",
        format!("Using builder: {:?}", builder_path).bright_black()
    );

    // Run the builder
    println!("\n{}", "Running builder...".bright_yellow());
    let status = Command::new(&builder_path)
        .arg(zip_path.to_string_lossy().as_ref())
        .status()
        .context("Failed to execute builder")?;

    if !status.success() {
        anyhow::bail!("Builder failed with exit code: {:?}", status.code());
    }

    println!(
        "\n{}",
        "✓ Docpack generation complete!".bright_green().bold()
    );
    println!(
        "{}",
        "Use 'localdoc list' to see installed docpacks".bright_black()
    );

    Ok(())
}

/// Find the builder binary in common locations
fn find_builder_binary() -> Result<PathBuf> {
    // Try common locations
    let candidates = vec![
        // Release build in builder directory
        PathBuf::from("../builder/target/release/doctown-builder"),
        PathBuf::from("builder/target/release/doctown-builder"),
        // Debug build in builder directory
        PathBuf::from("../builder/target/debug/doctown-builder"),
        PathBuf::from("builder/target/debug/doctown-builder"),
        // System path
        PathBuf::from("doctown-builder"),
    ];

    for candidate in candidates {
        // Check if it's an absolute or relative path that exists
        if candidate.exists() {
            return Ok(candidate);
        }

        // If it's just a binary name, check if it's in PATH
        if candidate.file_name().is_some() && candidate.parent().is_none() {
            if let Ok(output) = Command::new("which")
                .arg(candidate.to_string_lossy().as_ref())
                .output()
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Ok(PathBuf::from(path));
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Could not find builder binary. Please build it first:\n  \
        cd builder && cargo build --release"
    ))
}

/// Check if the input string is a GitHub URL
fn is_github_url(input: &str) -> bool {
    input.starts_with("http://github.com/")
        || input.starts_with("https://github.com/")
        || input.starts_with("http://www.github.com/")
        || input.starts_with("https://www.github.com/")
}

/// Parse GitHub URL and convert to zip download URL
fn parse_github_url(url: &str) -> Result<String> {
    let url = url.trim_end_matches('/');

    // Extract owner and repo from URL
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() < 5 {
        anyhow::bail!("Invalid GitHub URL format. Expected: https://github.com/owner/repo");
    }

    let owner = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];

    // Construct zip download URL for main branch
    Ok(format!(
        "https://github.com/{}/{}/archive/refs/heads/main.zip",
        owner, repo
    ))
}

/// Download a GitHub repository as a zip file
fn download_github_repo(url: &str) -> Result<PathBuf> {
    println!("{}", format!("Downloading from: {}", url).bright_black());

    let zip_url = parse_github_url(url)?;
    println!("{}", format!("Fetching: {}", zip_url).bright_black());

    // Download the zip file
    let response = reqwest::blocking::get(&zip_url).context("Failed to download repository")?;

    if !response.status().is_success() {
        // Try 'master' branch if 'main' fails
        let zip_url_master = zip_url.replace("/main.zip", "/master.zip");
        println!(
            "{}",
            format!("Main branch not found, trying master branch...").bright_yellow()
        );

        let response =
            reqwest::blocking::get(&zip_url_master).context("Failed to download repository")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Failed to download repository. Status: {}. \
                 Make sure the repository is public and accessible.",
                response.status()
            );
        }

        return download_and_save_zip(response);
    }

    download_and_save_zip(response)
}

/// Save the downloaded zip to a temporary file
fn download_and_save_zip(response: reqwest::blocking::Response) -> Result<PathBuf> {
    let bytes = response.bytes().context("Failed to read response body")?;

    // Create a temporary file
    let temp_file = tempfile::Builder::new()
        .prefix("github-repo-")
        .suffix(".zip")
        .tempfile()
        .context("Failed to create temporary file")?;

    // Write the zip content to the temp file
    std::fs::write(temp_file.path(), &bytes).context("Failed to write zip file")?;

    println!(
        "{}",
        format!("✓ Downloaded {} bytes", bytes.len()).bright_green()
    );

    // Convert to PathBuf (keeping the temp file alive by persisting it)
    let (_, path) = temp_file
        .keep()
        .context("Failed to persist temporary file")?;
    Ok(path)
}
