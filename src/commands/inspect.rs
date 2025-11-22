use crate::types::{EdgeKind, NodeKind};
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::PathBuf;

pub fn run(docpack: PathBuf, node_id: String) -> Result<()> {
    let (graph, _metadata, _documentation) = super::load_docpack(&docpack)?;

    let node = match graph.nodes.get(&node_id) {
        Some(n) => n,
        None => bail!("Node '{}' not found in graph", node_id),
    };

    println!(
        "\n{}",
        format!("Node: {}", node.name()).bright_cyan().bold()
    );
    println!("{}", "=".repeat(80).bright_black());

    println!("\n{}", "Basic Info".bright_green());
    println!("  ID:         {}", node.id.bright_white());
    println!("  Kind:       {}", node.kind_str());
    println!(
        "  Public:     {}",
        if node.is_public() {
            "yes".bright_green()
        } else {
            "no".bright_black()
        }
    );
    println!(
        "  Location:   {}:{}:{}",
        node.location.file, node.location.start_line, node.location.start_col
    );

    match &node.kind {
        NodeKind::Function(f) => {
            println!("\n{}", "Function Details".bright_green());
            println!("  Signature:  {}", f.signature);
            println!("  Async:      {}", f.is_async);
            println!("  Method:     {}", f.is_method);

            if !f.parameters.is_empty() {
                println!("\n  Parameters:");
                for param in &f.parameters {
                    let param_type = param.param_type.as_deref().unwrap_or("?");
                    let mutable = if param.is_mutable { "mut " } else { "" };
                    println!("    {}{}: {}", mutable, param.name, param_type);
                }
            }

            if let Some(ref ret) = f.return_type {
                println!("  Returns:    {}", ret);
            }
        }
        NodeKind::Type(t) => {
            println!("\n{}", "Type Details".bright_green());
            println!("  Type Kind:  {:?}", t.kind);

            if !t.fields.is_empty() {
                println!("\n  Fields:");
                for field in &t.fields {
                    let vis = if field.is_public { "pub" } else { "priv" };
                    let field_type = field.field_type.as_deref().unwrap_or("?");
                    println!("    {} {}: {}", vis, field.name, field_type);
                }
            }

            if !t.methods.is_empty() {
                println!("\n  Methods: {} method(s)", t.methods.len());
            }
        }
        NodeKind::Module(m) => {
            println!("\n{}", "Module Details".bright_green());
            println!("  Path:       {}", m.path);
            println!("  Children:   {}", m.children.len());
        }
        NodeKind::File(f) => {
            println!("\n{}", "File Details".bright_green());
            println!("  Language:   {}", f.language);
            println!("  Size:       {} bytes", f.size_bytes);
            println!("  Lines:      {}", f.line_count);
            println!("  Symbols:    {}", f.symbols.len());
        }
        NodeKind::Cluster(c) => {
            println!("\n{}", "Cluster Details".bright_green());
            if let Some(ref topic) = c.topic {
                println!("  Topic:      {}", topic);
            }
            println!("  Members:    {}", c.members.len());

            if !c.keywords.is_empty() {
                println!("  Keywords:   {}", c.keywords.join(", "));
            }
        }
        NodeKind::Constant(c) => {
            println!("\n{}", "Constant Details".bright_green());
            if let Some(ref vtype) = c.value_type {
                println!("  Type:       {}", vtype);
            }
        }
        NodeKind::Trait(t) => {
            println!("\n{}", "Trait Details".bright_green());
            println!("  Methods:    {}", t.methods.len());
            println!("  Implementors: {}", t.implementors.len());
        }
        NodeKind::Macro(m) => {
            println!("\n{}", "Macro Details".bright_green());
            println!("  Type:       {:?}", m.macro_type);
            if let Some(ref pattern) = m.pattern {
                println!("  Pattern:    {}", pattern);
            }
        }
        NodeKind::Package(p) => {
            println!("\n{}", "Package Details".bright_green());
            if let Some(ref version) = p.version {
                println!("  Version:    {}", version);
            }
            println!("  Modules:    {}", p.modules.len());
        }
    }

    println!("\n{}", "Metadata".bright_green());
    if let Some(complexity) = node.metadata.complexity {
        println!("  Complexity: {}", complexity);
    }
    println!(
        "  Fan-in:     {} (depended upon by {} nodes)",
        node.metadata.fan_in, node.metadata.fan_in
    );
    println!(
        "  Fan-out:    {} (depends on {} nodes)",
        node.metadata.fan_out, node.metadata.fan_out
    );
    println!("  Public API: {}", node.metadata.is_public_api);

    if let Some(ref docstring) = node.metadata.docstring {
        println!("\n{}", "Documentation".bright_green());
        println!("  {}", docstring);
    }

    if !node.metadata.tags.is_empty() {
        println!("\n{}", "Tags".bright_green());
        println!("  {}", node.metadata.tags.join(", "));
    }

    let outgoing: Vec<_> = graph.edges.iter().filter(|e| e.source == node.id).collect();

    let incoming: Vec<_> = graph.edges.iter().filter(|e| e.target == node.id).collect();

    if !outgoing.is_empty() {
        println!("\n{}", "Outgoing Edges".bright_green());

        let mut by_kind: std::collections::HashMap<&EdgeKind, Vec<&str>> =
            std::collections::HashMap::new();
        for edge in &outgoing {
            by_kind
                .entry(&edge.kind)
                .or_insert_with(Vec::new)
                .push(&edge.target);
        }

        for (kind, targets) in by_kind.iter() {
            println!("  {:?}: {} target(s)", kind, targets.len());
            for target in targets.iter().take(3) {
                if let Some(target_node) = graph.nodes.get(*target) {
                    println!("    -> {}", target_node.name().bright_black());
                }
            }
            if targets.len() > 3 {
                println!("    ... and {} more", targets.len() - 3);
            }
        }
    }

    if !incoming.is_empty() {
        println!("\n{}", "Incoming Edges".bright_green());

        let mut by_kind: std::collections::HashMap<&EdgeKind, Vec<&str>> =
            std::collections::HashMap::new();
        for edge in &incoming {
            by_kind
                .entry(&edge.kind)
                .or_insert_with(Vec::new)
                .push(&edge.source);
        }

        for (kind, sources) in by_kind.iter() {
            println!("  {:?}: {} source(s)", kind, sources.len());
            for source in sources.iter().take(3) {
                if let Some(source_node) = graph.nodes.get(*source) {
                    println!("    <- {}", source_node.name().bright_black());
                }
            }
            if sources.len() > 3 {
                println!("    ... and {} more", sources.len() - 3);
            }
        }
    }

    if let Some(ref snippet) = node.metadata.source_snippet {
        println!("\n{}", "Source Snippet".bright_green());
        println!("{}", snippet.bright_black());
    }

    println!();
    Ok(())
}
