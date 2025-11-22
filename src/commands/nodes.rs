use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(
    docpack: PathBuf,
    kind_filter: Option<String>,
    public_only: bool,
    limit: usize,
) -> Result<()> {
    let (graph, _metadata, _documentation) = super::load_docpack(&docpack)?;

    let mut nodes: Vec<_> = graph.nodes.values().collect();

    if let Some(ref kind) = kind_filter {
        let kind_lower = kind.to_lowercase();
        nodes.retain(|n| n.kind_str() == kind_lower);
    }

    if public_only {
        nodes.retain(|n| n.is_public());
    }

    nodes.sort_by(|a, b| a.name().cmp(&b.name()));

    let total = nodes.len();
    let showing = limit.min(total);

    println!(
        "\n{}",
        format!("Showing {} of {} nodes", showing, total)
            .bright_cyan()
            .bold()
    );

    if kind_filter.is_some() || public_only {
        let mut filters = Vec::new();
        if let Some(ref k) = kind_filter {
            filters.push(format!("kind={}", k));
        }
        if public_only {
            filters.push("public".to_string());
        }
        println!(
            "{}",
            format!("Filters: {}", filters.join(", ")).bright_black()
        );
    }

    println!("{}", "=".repeat(80).bright_black());

    for node in nodes.iter().take(limit) {
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
            "{} {:<10} {} {}",
            visibility,
            kind_colored,
            node.name().bright_white(),
            format!("@ {}:{}", node.location.file, node.location.start_line).bright_black()
        );

        if node.metadata.complexity.is_some() || node.metadata.fan_in > 0 {
            let mut metrics = Vec::new();

            if let Some(complexity) = node.metadata.complexity {
                metrics.push(format!("complexity={}", complexity));
            }
            if node.metadata.fan_in > 0 {
                metrics.push(format!("fan-in={}", node.metadata.fan_in));
            }
            if node.metadata.fan_out > 0 {
                metrics.push(format!("fan-out={}", node.metadata.fan_out));
            }

            if !metrics.is_empty() {
                println!("       {}", metrics.join(", ").bright_black());
            }
        }
    }

    if total > limit {
        println!(
            "\n{}",
            format!(
                "... and {} more nodes (use --limit to show more)",
                total - limit
            )
            .bright_black()
        );
    }

    println!();
    Ok(())
}
