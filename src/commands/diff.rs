use crate::commands::load_docpack;
use crate::types::{DocpackGraph, Documentation, Node, NodeId, NodeKind};
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::path::Path;

pub fn run(old_path: impl AsRef<Path>, new_path: impl AsRef<Path>) -> Result<()> {
    let (old_graph, _old_metadata, old_docs) = load_docpack(&old_path)?;
    let (new_graph, _new_metadata, new_docs) = load_docpack(&new_path)?;

    println!("📊 Docpack Comparison");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // Node additions and removals
    let (added, removed, common) = compute_node_diff(&old_graph, &new_graph);

    println!("📦 Node Changes:");
    println!("  ✨ Added:   {} nodes", added.len());
    println!("  🗑️  Removed: {} nodes", removed.len());
    println!("  🔄 Common:  {} nodes\n", common.len());

    if !added.is_empty() {
        println!("  Added nodes:");
        for (_node_id, node) in added.iter().take(10) {
            println!("    + {} [{}]", node.name(), node.kind_str());
        }
        if added.len() > 10 {
            println!("    ... and {} more", added.len() - 10);
        }
        println!();
    }

    if !removed.is_empty() {
        println!("  Removed nodes:");
        for (_node_id, node) in removed.iter().take(10) {
            println!("    - {} [{}]", node.name(), node.kind_str());
        }
        if removed.len() > 10 {
            println!("    ... and {} more", removed.len() - 10);
        }
        println!();
    }

    // Signature changes
    let signature_changes = detect_signature_changes(&old_graph, &new_graph, &common);
    if !signature_changes.is_empty() {
        println!("✏️  Signature Changes: {}", signature_changes.len());
        for change in signature_changes.iter().take(10) {
            println!("  📝 {}", change.node_name);
            println!("     Old: {}", change.old_signature);
            println!("     New: {}", change.new_signature);
        }
        if signature_changes.len() > 10 {
            println!("  ... and {} more\n", signature_changes.len() - 10);
        } else {
            println!();
        }
    }

    // Complexity deltas
    let complexity_deltas = compute_complexity_deltas(&old_graph, &new_graph, &common);
    if !complexity_deltas.is_empty() {
        let increased: Vec<_> = complexity_deltas.iter().filter(|d| d.delta > 0).collect();
        let decreased: Vec<_> = complexity_deltas.iter().filter(|d| d.delta < 0).collect();

        println!("🧮 Complexity Changes:");
        println!("  📈 Increased: {} nodes", increased.len());
        println!("  📉 Decreased: {} nodes", decreased.len());

        if !increased.is_empty() {
            println!("\n  Top complexity increases:");
            let mut sorted = increased.clone();
            sorted.sort_by_key(|d| -d.delta);
            for delta in sorted.iter().take(5) {
                println!(
                    "    {} [{}]: {} → {} (+{})",
                    delta.node_name,
                    delta.node_kind,
                    delta.old_complexity,
                    delta.new_complexity,
                    delta.delta
                );
            }
        }

        if !decreased.is_empty() {
            println!("\n  Top complexity decreases:");
            let mut sorted = decreased.clone();
            sorted.sort_by_key(|d| d.delta);
            for delta in sorted.iter().take(5) {
                println!(
                    "    {} [{}]: {} → {} ({})",
                    delta.node_name,
                    delta.node_kind,
                    delta.old_complexity,
                    delta.new_complexity,
                    delta.delta
                );
            }
        }
        println!();
    }

    // Semantic cluster drift
    if let (Some(ref old_doc), Some(ref new_doc)) = (old_docs, new_docs) {
        let cluster_drift = detect_cluster_drift(old_doc, new_doc, &common);
        if !cluster_drift.is_empty() {
            println!(
                "🎯 Semantic Cluster Drift: {} nodes changed clusters",
                cluster_drift.len()
            );
            for drift in cluster_drift.iter().take(10) {
                let old_cluster = drift.old_cluster.as_deref().unwrap_or("none");
                let new_cluster = drift.new_cluster.as_deref().unwrap_or("none");
                println!(
                    "  {} [{}]: \"{}\" → \"{}\"",
                    drift.node_name, drift.node_kind, old_cluster, new_cluster
                );
            }
            if cluster_drift.len() > 10 {
                println!("  ... and {} more\n", cluster_drift.len() - 10);
            } else {
                println!();
            }
        }

        // Documentation changes due to meaning changes
        let doc_changes = detect_meaningful_doc_changes(old_doc, new_doc, &common);
        if !doc_changes.is_empty() {
            println!(
                "📚 Documentation Changed (meaning shifted): {}",
                doc_changes.len()
            );
            for change in doc_changes.iter().take(5) {
                println!("  📖 {} [{}]", change.node_name, change.node_kind);
                println!("     Reason: {}", change.reason);
            }
            if doc_changes.len() > 5 {
                println!("  ... and {} more\n", doc_changes.len() - 5);
            } else {
                println!();
            }
        }
    }

    // Graph structure changes
    let structure_changes = analyze_graph_structure(&old_graph, &new_graph);
    if structure_changes.has_significant_changes() {
        println!("🌳 Graph Structure Changes:");
        println!(
            "  Edges: {} → {} (Δ {})",
            structure_changes.old_edge_count,
            structure_changes.new_edge_count,
            structure_changes.new_edge_count as i64 - structure_changes.old_edge_count as i64
        );

        if !structure_changes.heavily_mutated_subtrees.is_empty() {
            println!("\n  🔥 Heavily mutated subtrees:");
            for subtree in structure_changes.heavily_mutated_subtrees.iter().take(5) {
                println!("    {} - {} changes", subtree.root, subtree.change_count);
            }
        }
        println!();
    }

    // Summary
    let total_changes = added.len()
        + removed.len()
        + signature_changes.len()
        + complexity_deltas.len()
        + structure_changes.edge_delta().abs() as usize;

    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📊 Total changes detected: {}", total_changes);

    Ok(())
}

fn compute_node_diff<'a>(
    old_graph: &'a DocpackGraph,
    new_graph: &'a DocpackGraph,
) -> (
    HashMap<&'a NodeId, &'a Node>,
    HashMap<&'a NodeId, &'a Node>,
    Vec<&'a NodeId>,
) {
    let old_ids: HashSet<_> = old_graph.nodes.keys().collect();
    let new_ids: HashSet<_> = new_graph.nodes.keys().collect();

    let added: HashMap<_, _> = new_ids
        .difference(&old_ids)
        .map(|&id| (id, &new_graph.nodes[id]))
        .collect();

    let removed: HashMap<_, _> = old_ids
        .difference(&new_ids)
        .map(|&id| (id, &old_graph.nodes[id]))
        .collect();

    let common: Vec<_> = old_ids.intersection(&new_ids).copied().collect();

    (added, removed, common)
}

#[derive(Debug)]
struct SignatureChange {
    node_name: String,
    #[allow(dead_code)]
    node_kind: String,
    old_signature: String,
    new_signature: String,
}

fn detect_signature_changes(
    old_graph: &DocpackGraph,
    new_graph: &DocpackGraph,
    common: &[&NodeId],
) -> Vec<SignatureChange> {
    let mut changes = Vec::new();

    for node_id in common {
        let old_node = &old_graph.nodes[*node_id];
        let new_node = &new_graph.nodes[*node_id];

        let (old_sig, new_sig) = match (&old_node.kind, &new_node.kind) {
            (NodeKind::Function(old_f), NodeKind::Function(new_f)) => {
                if old_f.signature != new_f.signature {
                    (old_f.signature.clone(), new_f.signature.clone())
                } else {
                    continue;
                }
            }
            (NodeKind::Type(old_t), NodeKind::Type(new_t)) => {
                // Compare field signatures
                let old_sig = format!("{} with {} fields", old_t.name, old_t.fields.len());
                let new_sig = format!("{} with {} fields", new_t.name, new_t.fields.len());
                if old_t.fields != new_t.fields {
                    (old_sig, new_sig)
                } else {
                    continue;
                }
            }
            _ => continue,
        };

        changes.push(SignatureChange {
            node_name: old_node.name(),
            node_kind: old_node.kind_str().to_string(),
            old_signature: old_sig,
            new_signature: new_sig,
        });
    }

    changes
}

#[derive(Debug, Clone)]
struct ComplexityDelta {
    node_name: String,
    node_kind: String,
    old_complexity: u32,
    new_complexity: u32,
    delta: i32,
}

fn compute_complexity_deltas(
    old_graph: &DocpackGraph,
    new_graph: &DocpackGraph,
    common: &[&NodeId],
) -> Vec<ComplexityDelta> {
    let mut deltas = Vec::new();

    for node_id in common {
        let old_node = &old_graph.nodes[*node_id];
        let new_node = &new_graph.nodes[*node_id];

        if let (Some(old_complexity), Some(new_complexity)) =
            (old_node.metadata.complexity, new_node.metadata.complexity)
        {
            if old_complexity != new_complexity {
                deltas.push(ComplexityDelta {
                    node_name: old_node.name(),
                    node_kind: old_node.kind_str().to_string(),
                    old_complexity,
                    new_complexity,
                    delta: new_complexity as i32 - old_complexity as i32,
                });
            }
        }
    }

    deltas
}

#[derive(Debug)]
struct ClusterDrift {
    node_name: String,
    node_kind: String,
    old_cluster: Option<String>,
    new_cluster: Option<String>,
}

fn detect_cluster_drift(
    old_docs: &Documentation,
    new_docs: &Documentation,
    common: &[&NodeId],
) -> Vec<ClusterDrift> {
    let mut drifts = Vec::new();

    for node_id in common {
        let old_doc = old_docs.symbol_summaries.get(*node_id);
        let new_doc = new_docs.symbol_summaries.get(*node_id);

        if let (Some(old_doc), Some(new_doc)) = (old_doc, new_doc) {
            if old_doc.semantic_cluster != new_doc.semantic_cluster {
                drifts.push(ClusterDrift {
                    node_name: node_id.to_string(),
                    node_kind: "symbol".to_string(),
                    old_cluster: old_doc.semantic_cluster.clone(),
                    new_cluster: new_doc.semantic_cluster.clone(),
                });
            }
        }
    }

    drifts
}

#[derive(Debug)]
struct DocChange {
    node_name: String,
    node_kind: String,
    reason: String,
}

fn detect_meaningful_doc_changes(
    old_docs: &Documentation,
    new_docs: &Documentation,
    common: &[&NodeId],
) -> Vec<DocChange> {
    let mut changes = Vec::new();

    for node_id in common {
        let old_doc = old_docs.symbol_summaries.get(*node_id);
        let new_doc = new_docs.symbol_summaries.get(*node_id);

        if let (Some(old_doc), Some(new_doc)) = (old_doc, new_doc) {
            // Check if the purpose or explanation changed significantly
            let purpose_changed = old_doc.purpose != new_doc.purpose;
            let explanation_changed = old_doc.explanation != new_doc.explanation;

            if purpose_changed || explanation_changed {
                let mut reasons = Vec::new();
                if purpose_changed {
                    reasons.push("purpose changed");
                }
                if explanation_changed {
                    reasons.push("explanation updated");
                }

                changes.push(DocChange {
                    node_name: node_id.to_string(),
                    node_kind: "symbol".to_string(),
                    reason: reasons.join(", "),
                });
            }
        }
    }

    changes
}

#[derive(Debug)]
struct StructureChanges {
    old_edge_count: usize,
    new_edge_count: usize,
    heavily_mutated_subtrees: Vec<SubtreeMutation>,
}

#[derive(Debug)]
struct SubtreeMutation {
    root: String,
    change_count: usize,
}

impl StructureChanges {
    fn edge_delta(&self) -> i64 {
        self.new_edge_count as i64 - self.old_edge_count as i64
    }

    fn has_significant_changes(&self) -> bool {
        self.edge_delta().abs() > 0 || !self.heavily_mutated_subtrees.is_empty()
    }
}

fn analyze_graph_structure(old_graph: &DocpackGraph, new_graph: &DocpackGraph) -> StructureChanges {
    let old_edge_count = old_graph.edges.len();
    let new_edge_count = new_graph.edges.len();

    // Detect heavily mutated subtrees by looking at modules with many changes
    let mut module_changes: HashMap<String, usize> = HashMap::new();

    let old_ids: HashSet<_> = old_graph.nodes.keys().collect();
    let new_ids: HashSet<_> = new_graph.nodes.keys().collect();

    // Count changes per module
    for node_id in old_ids.symmetric_difference(&new_ids) {
        // Extract module path from node_id (rough heuristic)
        if let Some(module) = extract_module_path(node_id) {
            *module_changes.entry(module).or_insert(0) += 1;
        }
    }

    let mut heavily_mutated_subtrees: Vec<_> = module_changes
        .into_iter()
        .filter(|(_, count)| *count >= 3) // Threshold for "heavy mutation"
        .map(|(root, change_count)| SubtreeMutation { root, change_count })
        .collect();

    heavily_mutated_subtrees.sort_by_key(|s| std::cmp::Reverse(s.change_count));

    StructureChanges {
        old_edge_count,
        new_edge_count,
        heavily_mutated_subtrees,
    }
}

fn extract_module_path(node_id: &str) -> Option<String> {
    // Extract the file path portion before :: markers
    if let Some(first_sep) = node_id.find("::") {
        Some(node_id[..first_sep].to_string())
    } else {
        None
    }
}
