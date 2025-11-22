use anyhow::{bail, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn run(docpack: PathBuf, node_id: String) -> Result<()> {
    let (graph, _metadata, documentation) = super::load_docpack(&docpack)?;

    let node = match graph.nodes.get(&node_id) {
        Some(n) => n,
        None => bail!("Node '{}' not found in graph", node_id),
    };

    println!(
        "\n{}",
        format!("Documentation for: {}", node.name())
            .bright_cyan()
            .bold()
    );
    println!("{}", "=".repeat(80).bright_black());

    println!("\n{}", "Node Info".bright_green());
    println!("  ID:         {}", node.id);
    println!("  Kind:       {}", node.kind_str());
    println!(
        "  Location:   {}:{}",
        node.location.file, node.location.start_line
    );

    if let Some(ref docstring) = node.metadata.docstring {
        println!("\n{}", "Inline Documentation".bright_green());
        println!("{}", docstring);
    }

    if let Some(docs) = documentation {
        if let Some(symbol_doc) = docs.symbol_summaries.get(&node_id) {
            println!("\n{}", "AI-Generated Documentation".bright_green());

            println!("\n{}", "Purpose:".bright_yellow());
            println!("{}", symbol_doc.purpose);

            if !symbol_doc.explanation.is_empty() {
                println!("\n{}", "Explanation:".bright_yellow());
                println!("{}", symbol_doc.explanation);
            }

            if let Some(ref complexity) = symbol_doc.complexity_notes {
                println!("\n{}", "Complexity Notes:".bright_yellow());
                println!("{}", complexity);
            }

            if let Some(ref hints) = symbol_doc.usage_hints {
                println!("\n{}", "Usage Hints:".bright_yellow());
                println!("{}", hints);
            }

            if !symbol_doc.caller_references.is_empty() {
                println!("\n{}", "Caller References:".bright_yellow());
                for caller in &symbol_doc.caller_references {
                    println!("  - {}", caller);
                }
            }

            if !symbol_doc.callee_references.is_empty() {
                println!("\n{}", "Callee References:".bright_yellow());
                for callee in &symbol_doc.callee_references {
                    println!("  - {}", callee);
                }
            }

            if let Some(ref cluster) = symbol_doc.semantic_cluster {
                println!("\n{}", "Semantic Cluster:".bright_yellow());
                println!("{}", cluster);
            }
        } else {
            println!(
                "\n{}",
                "No AI-generated documentation available for this node".bright_yellow()
            );
        }
    } else {
        println!(
            "\n{}",
            "No documentation included in this docpack".bright_yellow()
        );
        println!(
            "{}",
            "The docpack may have been built without LLM documentation generation".bright_black()
        );
    }

    if let Some(ref snippet) = node.metadata.source_snippet {
        println!("\n{}", "Source Code".bright_green());
        println!("{}", snippet.bright_black());
    }

    println!();
    Ok(())
}
