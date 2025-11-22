use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

pub fn run(docpack: PathBuf) -> Result<()> {
    let (graph, _metadata, _documentation) = super::load_docpack(&docpack)?;

    println!("\n{}", "Detailed Statistics".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_black());

    println!("\n{}", "Node Counts".bright_green());
    let mut kind_counts: HashMap<&str, usize> = HashMap::new();
    for node in graph.nodes.values() {
        *kind_counts.entry(node.kind_str()).or_insert(0) += 1;
    }

    let mut kinds: Vec<_> = kind_counts.iter().collect();
    kinds.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    for (kind, count) in kinds {
        println!("  {:<12} {}", kind, count);
    }

    println!("\n{}", "Edge Counts".bright_green());
    let mut edge_counts: HashMap<String, usize> = HashMap::new();
    for edge in &graph.edges {
        let kind_str = format!("{:?}", edge.kind);
        *edge_counts.entry(kind_str).or_insert(0) += 1;
    }

    let mut edges: Vec<_> = edge_counts.iter().collect();
    edges.sort_by_key(|(_, count)| std::cmp::Reverse(**count));

    for (kind, count) in edges.iter().take(10) {
        println!("  {:<20} {}", kind, count);
    }

    if edges.len() > 10 {
        println!("  ... and {} more edge types", edges.len() - 10);
    }

    println!("\n{}", "Complexity Analysis".bright_green());
    let nodes_with_complexity: Vec<_> = graph
        .nodes
        .values()
        .filter_map(|n| n.metadata.complexity.map(|c| (n, c)))
        .collect();

    if !nodes_with_complexity.is_empty() {
        let total_complexity: u32 = nodes_with_complexity.iter().map(|(_, c)| c).sum();
        let avg_complexity = total_complexity as f64 / nodes_with_complexity.len() as f64;
        let max_complexity = nodes_with_complexity
            .iter()
            .map(|(_, c)| c)
            .max()
            .unwrap_or(&0);

        println!("  Nodes with complexity: {}", nodes_with_complexity.len());
        println!("  Average complexity:    {:.2}", avg_complexity);
        println!("  Max complexity:        {}", max_complexity);

        println!("\n  {}", "Most Complex Nodes:".bright_yellow());
        let mut by_complexity = nodes_with_complexity.clone();
        by_complexity.sort_by_key(|(_, c)| std::cmp::Reverse(*c));

        for (node, complexity) in by_complexity.iter().take(5) {
            println!(
                "    {} {} ({})",
                complexity.to_string().bright_red(),
                node.name(),
                node.location.file.bright_black()
            );
        }
    } else {
        println!("  No complexity data available");
    }

    println!("\n{}", "Fan-in/Fan-out Analysis".bright_green());
    let nodes_with_fanin: Vec<_> = graph
        .nodes
        .values()
        .filter(|n| n.metadata.fan_in > 0)
        .collect();

    if !nodes_with_fanin.is_empty() {
        let max_fan_in = nodes_with_fanin
            .iter()
            .map(|n| n.metadata.fan_in)
            .max()
            .unwrap_or(0);
        let max_fan_out = graph
            .nodes
            .values()
            .map(|n| n.metadata.fan_out)
            .max()
            .unwrap_or(0);

        println!("  Max fan-in:  {} (most depended upon)", max_fan_in);
        println!("  Max fan-out: {} (most dependencies)", max_fan_out);

        println!(
            "\n  {}",
            "Highest Fan-in (most depended upon):".bright_yellow()
        );
        let mut by_fanin: Vec<_> = graph.nodes.values().collect();
        by_fanin.sort_by_key(|n| std::cmp::Reverse(n.metadata.fan_in));

        for node in by_fanin.iter().take(5).filter(|n| n.metadata.fan_in > 0) {
            println!(
                "    {} {} ({})",
                node.metadata.fan_in.to_string().bright_cyan(),
                node.name(),
                node.kind_str().bright_black()
            );
        }
    } else {
        println!("  No fan-in/fan-out data available");
    }

    println!("\n{}", "Public API".bright_green());
    let public_api_nodes: Vec<_> = graph
        .nodes
        .values()
        .filter(|n| n.metadata.is_public_api)
        .collect();

    println!("  Public API nodes: {}", public_api_nodes.len());

    if !public_api_nodes.is_empty() {
        let mut api_by_kind: HashMap<&str, usize> = HashMap::new();
        for node in &public_api_nodes {
            *api_by_kind.entry(node.kind_str()).or_insert(0) += 1;
        }

        for (kind, count) in api_by_kind.iter() {
            println!("    {:<12} {}", kind, count);
        }
    }

    println!();
    Ok(())
}
