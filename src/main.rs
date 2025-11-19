mod docpack;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use docpack::Docpack;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "localdoc")]
#[command(about = "Query and inspect docpack documentation", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inspect top-level metadata of a docpack
    Inspect {
        /// Path or name (e.g., "xandwr:localdoc") of the docpack
        docpack: String,
    },
    /// Query docpack contents
    Query {
        /// Path or name (e.g., "xandwr:localdoc") of the docpack
        docpack: String,
        #[command(subcommand)]
        query_type: QueryType,
    },
    /// Install a docpack from the commons
    Install {
        /// Docpack identifier in format username:reponame
        package: String,
    },
    /// List installed docpacks
    List,
}

#[derive(Subcommand)]
enum QueryType {
    /// List all symbol names
    Symbols,
    /// Get full JSON entry for a specific symbol
    Symbol {
        /// Name or ID of the symbol to look up
        name: String,
    },
    /// Full-text search across summary/description
    Search {
        /// Keyword to search for
        keyword: String,
    },
    /// List all source files referenced in the docpack
    Files,
    /// Show symbols that originated from a specific file
    File {
        /// File path to filter by
        file: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Inspect { docpack } => {
            let path = resolve_docpack_path(&docpack)?;
            inspect_docpack(&path)?
        }
        Commands::Query {
            docpack,
            query_type,
        } => {
            let path = resolve_docpack_path(&docpack)?;
            handle_query(&path, query_type)?
        }
        Commands::Install { package } => install_docpack(&package)?,
        Commands::List => list_docpacks()?,
    }

    Ok(())
}

/// Get the directory where docpacks are installed
fn get_packages_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine user data directory"))?;
    Ok(data_dir.join("localdoc").join("packages"))
}

/// Resolve a docpack identifier to a file path.
/// Accepts either:
/// - A full file path (e.g., "/path/to/file.docpack")
/// - A name in format "username:reponame" (e.g., "xandwr:localdoc")
fn resolve_docpack_path(identifier: &str) -> Result<String> {
    // If it looks like a path (contains path separators or ends with .docpack), use it directly
    if identifier.contains('/') || identifier.contains('\\') || identifier.ends_with(".docpack") {
        return Ok(identifier.to_string());
    }

    // Otherwise, treat it as a name and look for it in the packages directory
    let packages_dir = get_packages_dir()?;
    let filename = format!("{}.docpack", identifier.replace(':', "_"));
    let path = packages_dir.join(&filename);

    if path.exists() {
        Ok(path.to_string_lossy().to_string())
    } else {
        anyhow::bail!(
            "Docpack '{}' not found. Expected at: {}\nRun 'localdoc list' to see installed docpacks, or 'localdoc install {}' to install it.",
            identifier,
            path.display(),
            identifier
        )
    }
}

/// List all installed docpacks
fn list_docpacks() -> Result<()> {
    let packages_dir = get_packages_dir()?;

    if !packages_dir.exists() {
        println!("{}", "No docpacks installed yet.".yellow());
        println!();
        println!("Install one with: {}", "localdoc install <username:reponame>".cyan());
        return Ok(());
    }

    let entries: Vec<_> = std::fs::read_dir(&packages_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "docpack")
                .unwrap_or(false)
        })
        .collect();

    if entries.is_empty() {
        println!("{}", "No docpacks installed yet.".yellow());
        println!();
        println!("Install one with: {}", "localdoc install <username:reponame>".cyan());
        return Ok(());
    }

    println!("{}", "Installed Docpacks".bold().cyan());
    println!("{}", "=".repeat(50));
    println!();

    for entry in &entries {
        let path = entry.path();
        let filename = path.file_stem().unwrap_or_default().to_string_lossy();

        // Convert filename back to name format (username_reponame -> username:reponame)
        let name = filename.replacen('_', ":", 1);

        // Try to read manifest for additional info
        match Docpack::open(&path.to_string_lossy()) {
            Ok(docpack) => {
                let manifest = &docpack.manifest;
                println!(
                    "{} {} {}",
                    name.green().bold(),
                    format!("v{}", manifest.project.version).dimmed(),
                    format!("({} symbols)", manifest.stats.symbols_extracted).dimmed()
                );
            }
            Err(_) => {
                println!("{} {}", name.green().bold(), "(unable to read metadata)".dimmed());
            }
        }
    }

    println!();
    println!("Total: {} docpack(s)", entries.len());
    println!();
    println!("{}", "Usage:".bold());
    println!("  {} {}", "localdoc inspect".dimmed(), "<name>".cyan());
    println!("  {} {} {}", "localdoc query".dimmed(), "<name>".cyan(), "symbols".dimmed());

    Ok(())
}

fn inspect_docpack(path: &str) -> Result<()> {
    let docpack = Docpack::open(path)?;
    let manifest = &docpack.manifest;

    println!("{}", "Docpack Metadata".bold().cyan());
    println!("{}", "=".repeat(50));
    println!();

    println!("{}: {}", "Format Version".bold(), manifest.docpack_format);
    println!();

    println!("{}", "Project Information:".bold().green());
    println!("  {}: {}", "Name".bold(), manifest.project.name);
    println!("  {}: {}", "Version".bold(), manifest.project.version);
    if !manifest.project.repo.is_empty() {
        println!("  {}: {}", "Repository".bold(), manifest.project.repo);
    }
    if !manifest.project.commit.is_empty() {
        println!("  {}: {}", "Commit".bold(), manifest.project.commit);
    }
    println!();

    println!("{}: {}", "Generated At".bold(), manifest.generated_at);
    println!();

    println!("{}", "Language Summary:".bold().yellow());
    for (lang, count) in &manifest.language_summary {
        println!("  {}: {}", lang, count);
    }
    println!();

    println!("{}", "Statistics:".bold().magenta());
    println!(
        "  {}: {}",
        "Symbols Extracted".bold(),
        manifest.stats.symbols_extracted
    );
    println!(
        "  {}: {}",
        "Docs Generated".bold(),
        manifest.stats.docs_generated
    );

    Ok(())
}

fn handle_query(path: &str, query_type: QueryType) -> Result<()> {
    let mut docpack = Docpack::open(path)?;

    match query_type {
        QueryType::Symbols => {
            println!("{}", "All Symbols".bold().cyan());
            println!("{}", "=".repeat(50));
            println!();

            for symbol in &docpack.symbols {
                println!(
                    "{} {} {}",
                    format!("[{}]", symbol.kind).yellow(),
                    symbol.id.green(),
                    format!("({}:{})", symbol.file, symbol.line).dimmed()
                );
            }

            println!();
            println!("Total: {} symbols", docpack.symbols.len());
        }

        QueryType::Symbol { name } => {
            let matches: Vec<_> = docpack
                .find_symbols_by_name(&name)
                .into_iter()
                .cloned()
                .collect();

            if matches.is_empty() {
                eprintln!("{}", format!("No symbol found matching '{}'", name).red());
                std::process::exit(1);
            }

            for symbol in matches {
                let doc = docpack.get_documentation(&symbol.doc_id)?;

                println!("{}", "Symbol Information".bold().cyan());
                println!("{}", "=".repeat(50));
                println!();

                println!("{}: {}", "ID".bold(), symbol.id.green());
                println!("{}: {}", "Kind".bold(), symbol.kind.yellow());
                println!(
                    "{}: {}",
                    "File".bold(),
                    format!("{}:{}", symbol.file, symbol.line)
                );
                println!("{}: {}", "Signature".bold(), symbol.signature);
                println!();

                println!("{}", "Documentation".bold().cyan());
                println!("{}", "-".repeat(50));
                println!();
                println!("{}: {}", "Summary".bold(), doc.summary);
                println!();
                println!("{}", "Description:".bold());
                println!("{}", doc.description);
                println!();

                if !doc.parameters.is_empty() {
                    println!("{}", "Parameters:".bold().green());
                    for param in &doc.parameters {
                        println!(
                            "  {} {} - {}",
                            param.name.bold(),
                            format!("({})", param.param_type).dimmed(),
                            param.description
                        );
                    }
                    println!();
                }

                println!("{}: {}", "Returns".bold(), doc.returns);
                println!();

                if !doc.example.is_empty() {
                    println!("{}", "Example:".bold().yellow());
                    println!("{}", doc.example);
                    println!();
                }

                if !doc.notes.is_empty() {
                    println!("{}", "Notes:".bold().magenta());
                    for note in &doc.notes {
                        println!("  - {}", note);
                    }
                    println!();
                }
            }
        }

        QueryType::Search { keyword } => {
            let results = docpack.search_symbols(&keyword)?;

            if results.is_empty() {
                eprintln!("{}", format!("No results found for '{}'", keyword).red());
                std::process::exit(1);
            }

            println!(
                "{}",
                format!("Search Results for '{}'", keyword).bold().cyan()
            );
            println!("{}", "=".repeat(50));
            println!();

            for (symbol, doc) in results {
                println!(
                    "{} {}",
                    format!("[{}]", symbol.kind).yellow(),
                    symbol.id.green()
                );
                println!(
                    "  {}: {}",
                    "Location".dimmed(),
                    format!("{}:{}", symbol.file, symbol.line).dimmed()
                );
                println!("  {}: {}", "Summary".bold(), doc.summary);
                println!();
            }
        }

        QueryType::Files => {
            let files = docpack.get_unique_files();

            println!("{}", "Source Files".bold().cyan());
            println!("{}", "=".repeat(50));
            println!();

            for file in &files {
                let count = docpack.symbols.iter().filter(|s| &s.file == file).count();
                println!(
                    "{} {}",
                    file.green(),
                    format!("({} symbols)", count).dimmed()
                );
            }

            println!();
            println!("Total: {} files", files.len());
        }

        QueryType::File { file } => {
            let symbols = docpack.find_symbols_by_file(&file);

            if symbols.is_empty() {
                eprintln!(
                    "{}",
                    format!("No symbols found in file matching '{}'", file).red()
                );
                std::process::exit(1);
            }

            println!("{}", format!("Symbols in '{}'", file).bold().cyan());
            println!("{}", "=".repeat(50));
            println!();

            for symbol in symbols {
                println!(
                    "{} {} {}",
                    format!("[{}]", symbol.kind).yellow(),
                    symbol.id.green(),
                    format!("(line {})", symbol.line).dimmed()
                );
                println!("  {}", symbol.signature.dimmed());
                println!();
            }
        }
    }

    Ok(())
}

fn install_docpack(package: &str) -> Result<()> {
    use std::fs;
    use std::io::Write;

    println!("{}", format!("Installing {}...", package).bold().cyan());

    // Parse the package identifier (username:reponame)
    let full_name = package.replace(':', "/");

    // Get or create the localdoc directory in user's data directory
    let data_dir = dirs::data_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine user data directory"))?;
    let localdoc_dir = data_dir.join("localdoc").join("packages");

    fs::create_dir_all(&localdoc_dir)?;

    // Fetch the docpack list from the commons API
    // Use environment variable if set, otherwise use default production URL
    let api_url = std::env::var("DOCTOWN_API_URL")
        .unwrap_or_else(|_| "https://www.doctown.dev/api/docpacks?public=true".to_string());

    println!("{}", format!("Fetching from {}...", api_url).dimmed());

    let response = reqwest::blocking::get(api_url)
        .map_err(|e| anyhow::anyhow!("Failed to fetch from commons: {}", e))?;

    if !response.status().is_success() {
        anyhow::bail!("API request failed with status: {}", response.status());
    }

    let response_text = response.text()
        .map_err(|e| anyhow::anyhow!("Failed to read response text: {}", e))?;

    let body: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| anyhow::anyhow!("Failed to parse API response: {}. Response body: {}", e, response_text))?;

    let docpacks = body["docpacks"]
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("Invalid API response format. Body: {}", body))?;

    // Debug: show available docpacks if LOCALDOC_DEBUG is set
    if std::env::var("LOCALDOC_DEBUG").is_ok() {
        eprintln!("Looking for: {}", full_name);
        eprintln!("Available docpacks:");
        for dp in docpacks {
            eprintln!("  - {}: {}", dp["full_name"].as_str().unwrap_or("N/A"), dp["file_url"].as_str().unwrap_or("N/A"));
        }
    }

    // Find the matching docpack
    let docpack = docpacks
        .iter()
        .find(|d| d["full_name"].as_str() == Some(&full_name))
        .ok_or_else(|| {
            let available: Vec<_> = docpacks.iter()
                .filter_map(|d| d["full_name"].as_str())
                .collect();
            anyhow::anyhow!("Docpack '{}' (looking for '{}') not found in commons. Available: {:?}", package, full_name, available)
        })?;

    let file_url = docpack["file_url"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Docpack does not have a download URL"))?;

    // Download the docpack file
    println!("{}", format!("Downloading docpack from: {}...", file_url).dimmed());

    let file_response = reqwest::blocking::get(file_url)
        .map_err(|e| anyhow::anyhow!("Failed to download docpack: {}", e))?;

    let status = file_response.status();
    if !status.is_success() {
        let error_body = file_response.text().unwrap_or_else(|_| "Unable to read error body".to_string());
        anyhow::bail!("Download failed with status: {}. Error: {}", status, error_body);
    }

    let bytes = file_response.bytes()
        .map_err(|e| anyhow::anyhow!("Failed to read docpack data: {}", e))?;

    // Save the docpack file
    let filename = format!("{}.docpack", package.replace(':', "_"));
    let dest_path = localdoc_dir.join(&filename);

    let mut file = fs::File::create(&dest_path)?;
    file.write_all(&bytes)?;

    println!();
    println!("{}", "Installation complete!".green().bold());
    println!();
    println!("{}: {}", "Package".bold(), package.green());
    println!("{}: {}", "Location".bold(), dest_path.display().to_string().dimmed());
    println!();
    println!("{}", "Usage:".bold());
    println!(
        "  {} {} {}",
        "localdoc inspect".dimmed(),
        package.cyan(),
        "# View metadata".dimmed()
    );
    println!(
        "  {} {} {}",
        "localdoc query".dimmed(),
        package.cyan(),
        "symbols # List all symbols".dimmed()
    );

    Ok(())
}
