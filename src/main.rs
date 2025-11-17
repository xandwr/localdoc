mod docpack;
mod models;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;
use docpack::Docpack;

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
        /// Path to the docpack file
        docpack: String,
    },
    /// Query docpack contents
    Query {
        /// Path to the docpack file
        docpack: String,
        #[command(subcommand)]
        query_type: QueryType,
    },
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
        Commands::Inspect { docpack } => inspect_docpack(&docpack)?,
        Commands::Query {
            docpack,
            query_type,
        } => handle_query(&docpack, query_type)?,
    }

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
