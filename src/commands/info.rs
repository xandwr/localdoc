use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;

pub fn run(docpack: PathBuf) -> Result<()> {
    let (graph, metadata, documentation) = super::load_docpack(&docpack)?;

    println!("\n{}", "Docpack Info".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_black());

    println!("\n{}", "Package".bright_green());
    println!("  Source:     {}", metadata.source);
    println!("  Generated:  {}", metadata.generated_at);
    println!("  Generator:  {}", metadata.generator);
    println!("  Version:    {}", metadata.version);
    println!(
        "  Size:       {:.2} KB",
        metadata.total_size_bytes as f64 / 1024.0
    );

    println!("\n{}", "Graph Contents".bright_green());
    println!("  Total Nodes:   {}", graph.nodes.len());
    println!("  Total Edges:   {}", graph.edges.len());
    println!("  Files:         {}", graph.metadata.total_files);
    println!("  Symbols:       {}", graph.metadata.total_symbols);

    if !graph.metadata.languages.is_empty() {
        let langs: Vec<_> = graph
            .metadata
            .languages
            .iter()
            .map(|s| s.as_str())
            .collect();
        println!("  Languages:     {}", langs.join(", "));
    }

    if let Some(ref repo) = graph.metadata.repository_name {
        println!("  Repository:    {}", repo);
    }

    let function_count = graph
        .nodes
        .values()
        .filter(|n| n.kind_str() == "function")
        .count();
    let type_count = graph
        .nodes
        .values()
        .filter(|n| n.kind_str() == "type")
        .count();
    let module_count = graph
        .nodes
        .values()
        .filter(|n| n.kind_str() == "module")
        .count();
    let cluster_count = graph
        .nodes
        .values()
        .filter(|n| n.kind_str() == "cluster")
        .count();

    println!("\n{}", "Breakdown".bright_green());
    println!("  Functions:     {}", function_count);
    println!("  Types:         {}", type_count);
    println!("  Modules:       {}", module_count);
    println!("  Clusters:      {}", cluster_count);

    if let Some(docs) = documentation {
        println!("\n{}", "Documentation".bright_green());
        println!("  Symbol docs:   {}", docs.symbol_summaries.len());
        println!("  Module docs:   {}", docs.module_overviews.len());
        println!("  Tokens used:   {}", docs.total_tokens_used);
    } else {
        println!("\n{}", "Documentation".bright_yellow());
        println!("  No documentation included");
    }

    println!();
    Ok(())
}
