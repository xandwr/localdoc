use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(docpack: PathBuf, query: String, case_sensitive: bool) -> Result<()> {
    let (graph, _metadata, _documentation) = super::load_docpack(&docpack)?;

    let query_lower = query.to_lowercase();

    let results: Vec<_> = graph
        .nodes
        .values()
        .filter(|node| {
            let name = node.name();
            if case_sensitive {
                name.contains(&query)
            } else {
                name.to_lowercase().contains(&query_lower)
            }
        })
        .collect();

    println!(
        "\n{}",
        format!("Found {} matching nodes", results.len())
            .bright_cyan()
            .bold()
    );
    println!("{}", format!("Query: '{}'", query).bright_black());
    println!("{}", "=".repeat(80).bright_black());

    if results.is_empty() {
        println!("\nNo nodes found matching '{}'", query);
        println!("\nTry:");
        println!("  - Using a shorter search term");
        println!("  - Checking your spelling");
        println!("  - Using case-insensitive search (default)");
        println!();
        return Ok(());
    }

    for node in results.iter().take(50) {
        let kind_str = node.kind_str();
        let kind_colored = match kind_str {
            "function" => kind_str.bright_blue(),
            "type" => kind_str.bright_green(),
            "module" => kind_str.bright_magenta(),
            "file" => kind_str.bright_yellow(),
            "cluster" => kind_str.bright_cyan(),
            _ => kind_str.white(),
        };

        let visibility = if node.is_public() {
            "pub".bright_green()
        } else {
            "priv".bright_black()
        };

        println!(
            "{} {:<10} {}",
            visibility,
            kind_colored,
            node.name().bright_white()
        );

        println!(
            "       {} {}",
            format!("@ {}:{}", node.location.file, node.location.start_line).bright_black(),
            format!("ID: {}", node.id).bright_black()
        );
    }

    if results.len() > 50 {
        println!(
            "\n{}",
            format!("... and {} more results", results.len() - 50).bright_black()
        );
        println!(
            "{}",
            "Use a more specific query to narrow results".bright_black()
        );
    }

    println!();
    Ok(())
}
