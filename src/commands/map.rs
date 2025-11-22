use crate::types::NodeKind;
use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Characters for box drawing
const TOP_LEFT: &str = "╭";
const TOP_RIGHT: &str = "╮";
const BOTTOM_LEFT: &str = "╰";
const BOTTOM_RIGHT: &str = "╯";
const HORIZONTAL: &str = "─";
const VERTICAL: &str = "│";
const BRANCH: &str = "├";

/// Cluster info for visualization
struct ClusterInfo {
    id: String,
    name: String,
    keywords: Vec<String>,
    member_count: usize,
    functions: usize,
    types: usize,
    avg_complexity: f64,
    centroid: Option<Vec<f32>>,
}

pub fn run(docpack: PathBuf, compact: bool) -> Result<()> {
    let (graph, metadata, _documentation) = super::load_docpack(&docpack)?;

    // Gather cluster information
    let mut clusters: Vec<ClusterInfo> = Vec::new();

    for node in graph.nodes.values() {
        if let NodeKind::Cluster(cluster) = &node.kind {
            let mut functions = 0;
            let mut types = 0;
            let mut total_complexity = 0u32;
            let mut complexity_count = 0;

            for member_id in &cluster.members {
                if let Some(member) = graph.nodes.get(member_id) {
                    match &member.kind {
                        NodeKind::Function(_) => functions += 1,
                        NodeKind::Type(_) => types += 1,
                        _ => {}
                    }
                    if let Some(c) = member.metadata.complexity {
                        total_complexity += c;
                        complexity_count += 1;
                    }
                }
            }

            let avg_complexity = if complexity_count > 0 {
                total_complexity as f64 / complexity_count as f64
            } else {
                0.0
            };

            clusters.push(ClusterInfo {
                id: node.id.clone(),
                name: cluster.name.clone(),
                keywords: cluster.keywords.clone(),
                member_count: cluster.members.len(),
                functions,
                types,
                avg_complexity,
                centroid: cluster.centroid.clone(),
            });
        }
    }

    // Sort clusters by member count (largest first)
    clusters.sort_by_key(|c| std::cmp::Reverse(c.member_count));

    // Calculate graph-wide stats
    let total_nodes = graph.nodes.len();
    let total_edges = graph.edges.len();
    let total_clustered: usize = clusters.iter().map(|c| c.member_count).sum();

    // Print header
    println!();
    print_header("SEMANTIC SUBSYSTEM MAP", &metadata.source);

    // Print overview stats
    print_overview_box(
        clusters.len(),
        total_clustered,
        total_nodes,
        total_edges,
        &graph.metadata.languages,
    );

    if clusters.is_empty() {
        println!(
            "\n{}",
            "  No clusters found. Run the builder with embedding pipeline enabled.".bright_yellow()
        );
        return Ok(());
    }

    // Calculate cluster relationships (shared edges between cluster members)
    let cluster_relationships = compute_cluster_relationships(&graph, &clusters);

    // Print the cluster constellation
    println!();
    print_constellation(&clusters, &cluster_relationships, compact);

    // Print cluster details
    println!();
    print_cluster_details(&clusters, compact);

    // Print relationship matrix if there are relationships
    if !cluster_relationships.is_empty() && !compact {
        println!();
        print_relationship_matrix(&clusters, &cluster_relationships);
    }

    // Print embedding space visualization
    if clusters.iter().any(|c| c.centroid.is_some()) && !compact {
        println!();
        print_embedding_projection(&clusters);
    }

    println!();
    Ok(())
}

fn print_header(title: &str, source: &str) {
    let width = 70;
    let padding = (width - title.len() - 2) / 2;

    println!(
        "  {}{}{}",
        "▓".bright_magenta(),
        "▓".repeat(width).bright_magenta(),
        "▓".bright_magenta()
    );
    println!(
        "  {} {}{}{} {}",
        "▓".bright_magenta(),
        " ".repeat(padding),
        title.bright_white().bold(),
        " ".repeat(width - padding - title.len() - 2),
        "▓".bright_magenta()
    );
    println!(
        "  {}{}{}",
        "▓".bright_magenta(),
        "▓".repeat(width).bright_magenta(),
        "▓".bright_magenta()
    );

    let source_display = if source.len() > 60 {
        format!("...{}", &source[source.len() - 57..])
    } else {
        source.to_string()
    };
    println!("  {} {}", "Source:".bright_black(), source_display.cyan());
}

fn print_overview_box(
    cluster_count: usize,
    clustered_nodes: usize,
    total_nodes: usize,
    total_edges: usize,
    languages: &std::collections::HashSet<String>,
) {
    let coverage = if total_nodes > 0 {
        (clustered_nodes as f64 / total_nodes as f64 * 100.0) as usize
    } else {
        0
    };

    println!();
    println!(
        "  {}",
        "┌─────────────────────────────────────────────────────────────────────┐".bright_black()
    );
    println!(
        "  {}  {}  {}   {}  {}   {}  {}   {} {}  {}",
        VERTICAL.bright_black(),
        "◆".bright_cyan(),
        format!("{} Subsystems", cluster_count).bright_white(),
        "◇".bright_green(),
        format!("{} Nodes", total_nodes).bright_white(),
        "◈".bright_yellow(),
        format!("{} Edges", total_edges).bright_white(),
        "Coverage:".bright_black(),
        format!("{}%", coverage).bright_magenta(),
        VERTICAL.bright_black()
    );
    println!(
        "  {}  {} {}  {}",
        VERTICAL.bright_black(),
        "Languages:".bright_black(),
        languages
            .iter()
            .map(|l| l.bright_cyan().to_string())
            .collect::<Vec<_>>()
            .join(", "),
        VERTICAL.bright_black()
    );
    println!(
        "  {}",
        "└─────────────────────────────────────────────────────────────────────┘".bright_black()
    );
}

fn compute_cluster_relationships(
    graph: &crate::types::DocpackGraph,
    clusters: &[ClusterInfo],
) -> HashMap<(usize, usize), usize> {
    let mut relationships: HashMap<(usize, usize), usize> = HashMap::new();

    // Build member -> cluster index mapping
    let mut member_to_cluster: HashMap<&str, usize> = HashMap::new();
    for (idx, cluster) in clusters.iter().enumerate() {
        if let Some(node) = graph.nodes.get(&cluster.id) {
            if let NodeKind::Cluster(c) = &node.kind {
                for member_id in &c.members {
                    member_to_cluster.insert(member_id.as_str(), idx);
                }
            }
        }
    }

    // Count edges between clusters
    for edge in &graph.edges {
        if let (Some(&src_cluster), Some(&dst_cluster)) = (
            member_to_cluster.get(edge.source.as_str()),
            member_to_cluster.get(edge.target.as_str()),
        ) {
            if src_cluster != dst_cluster {
                let key = if src_cluster < dst_cluster {
                    (src_cluster, dst_cluster)
                } else {
                    (dst_cluster, src_cluster)
                };
                *relationships.entry(key).or_insert(0) += 1;
            }
        }
    }

    relationships
}

fn print_constellation(
    clusters: &[ClusterInfo],
    relationships: &HashMap<(usize, usize), usize>,
    compact: bool,
) {
    println!("  {}", "SUBSYSTEM CONSTELLATION".bright_cyan().bold());
    println!(
        "  {}",
        "Semantic clusters discovered via HDBSCAN + MiniLM embeddings"
            .bright_black()
            .italic()
    );
    println!();

    let display_count = if compact {
        clusters.len().min(8)
    } else {
        clusters.len().min(15)
    };

    // Calculate max size for scaling
    let max_size = clusters.iter().map(|c| c.member_count).max().unwrap_or(1);

    for (idx, cluster) in clusters.iter().take(display_count).enumerate() {
        // Calculate visual width based on member count
        let bar_width = ((cluster.member_count as f64 / max_size as f64) * 40.0) as usize;
        let bar_width = bar_width.max(3);

        // Color based on complexity
        let complexity_color = if cluster.avg_complexity > 10.0 {
            "bright_red"
        } else if cluster.avg_complexity > 5.0 {
            "bright_yellow"
        } else {
            "bright_green"
        };

        // Cluster icon based on size
        let icon = if cluster.member_count > 20 {
            "████"
        } else if cluster.member_count > 10 {
            "███"
        } else if cluster.member_count > 5 {
            "██"
        } else {
            "█"
        };

        // Print cluster bar
        let bar = "█".repeat(bar_width);
        let colored_bar = match complexity_color {
            "bright_red" => bar.bright_red(),
            "bright_yellow" => bar.bright_yellow(),
            _ => bar.bright_green(),
        };

        let name_display = if cluster.name.len() > 20 {
            format!("{}...", &cluster.name[..17])
        } else {
            cluster.name.clone()
        };

        println!(
            "  {:>2}. {} {:<20} {} {:>3} {}",
            idx + 1,
            icon.bright_cyan(),
            name_display.bright_white().bold(),
            colored_bar,
            cluster.member_count,
            "nodes".bright_black()
        );

        // Print keywords on the next line
        if !cluster.keywords.is_empty() && !compact {
            let keywords: String = cluster
                .keywords
                .iter()
                .take(5)
                .map(|k| format!("#{}", k))
                .collect::<Vec<_>>()
                .join(" ");
            println!("      {} {}", "↳".bright_black(), keywords.bright_blue());
        }

        // Show connections to other clusters
        let connections: Vec<_> = relationships
            .iter()
            .filter(|((a, b), _)| *a == idx || *b == idx)
            .collect();

        if !connections.is_empty() && !compact {
            let conn_str: String = connections
                .iter()
                .take(3)
                .map(|((a, b), count)| {
                    let other = if *a == idx { *b } else { *a };
                    format!("→#{} ({})", other + 1, count)
                })
                .collect::<Vec<_>>()
                .join(" ");
            println!(
                "        {} {}",
                "links:".bright_black(),
                conn_str.bright_magenta()
            );
        }
    }

    if clusters.len() > display_count {
        println!(
            "\n      {} {} more clusters...",
            "...".bright_black(),
            clusters.len() - display_count
        );
    }
}

fn print_cluster_details(clusters: &[ClusterInfo], compact: bool) {
    println!("  {}", "CLUSTER COMPOSITION".bright_yellow().bold());
    println!();

    let width = 70;

    // Table header
    println!(
        "  {} {:<20} {:>6} {:>6} {:>6} {:>10} {}",
        VERTICAL.bright_black(),
        "Cluster".bright_white().bold(),
        "Funcs".bright_cyan(),
        "Types".bright_green(),
        "Total".bright_white(),
        "Complexity".bright_yellow(),
        VERTICAL.bright_black()
    );
    println!(
        "  {}{}{}",
        BRANCH.bright_black(),
        HORIZONTAL.repeat(width - 2).bright_black(),
        "┤".bright_black()
    );

    let display_count = if compact { 8 } else { 15 };

    for cluster in clusters.iter().take(display_count) {
        let name_display = if cluster.name.len() > 18 {
            format!("{}...", &cluster.name[..15])
        } else {
            cluster.name.clone()
        };

        let complexity_str = format!("{:.1}", cluster.avg_complexity);
        let complexity_colored = if cluster.avg_complexity > 10.0 {
            complexity_str.bright_red()
        } else if cluster.avg_complexity > 5.0 {
            complexity_str.bright_yellow()
        } else {
            complexity_str.bright_green()
        };

        println!(
            "  {} {:<20} {:>6} {:>6} {:>6} {:>10} {}",
            VERTICAL.bright_black(),
            name_display.bright_white(),
            cluster.functions.to_string().bright_cyan(),
            cluster.types.to_string().bright_green(),
            cluster.member_count.to_string().bright_white(),
            complexity_colored,
            VERTICAL.bright_black()
        );
    }

    println!(
        "  {}{}{}",
        BOTTOM_LEFT.bright_black(),
        HORIZONTAL.repeat(width - 2).bright_black(),
        BOTTOM_RIGHT.bright_black()
    );
}

fn print_relationship_matrix(
    clusters: &[ClusterInfo],
    relationships: &HashMap<(usize, usize), usize>,
) {
    let display_count = clusters.len().min(10);

    println!(
        "  {}",
        "INTER-CLUSTER RELATIONSHIPS".bright_magenta().bold()
    );
    println!(
        "  {}",
        "Edge counts between semantic subsystems"
            .bright_black()
            .italic()
    );
    println!();

    // Print column headers
    print!("  {:>8} ", "");
    for i in 0..display_count {
        print!("{:>4}", format!("#{}", i + 1).bright_cyan());
    }
    println!();

    // Print matrix rows
    for i in 0..display_count {
        let name = if clusters[i].name.len() > 6 {
            format!("{}.", &clusters[i].name[..5])
        } else {
            clusters[i].name.clone()
        };
        print!("  {:>6} {} ", name.bright_white(), VERTICAL.bright_black());

        for j in 0..display_count {
            if i == j {
                print!("{:>4}", "●".bright_black());
            } else {
                let key = if i < j { (i, j) } else { (j, i) };
                let count = relationships.get(&key).unwrap_or(&0);
                if *count > 0 {
                    let intensity = if *count > 20 {
                        "███".bright_red()
                    } else if *count > 10 {
                        "██".bright_yellow()
                    } else if *count > 5 {
                        "█".bright_green()
                    } else {
                        "·".bright_blue()
                    };
                    print!("{:>4}", intensity);
                } else {
                    print!("{:>4}", "·".bright_black());
                }
            }
        }
        println!();
    }

    println!();
    println!(
        "  {} {} {} {} {} {} {} {}",
        "Legend:".bright_black(),
        "·".bright_blue(),
        "1-5".bright_black(),
        "█".bright_green(),
        "6-10".bright_black(),
        "██".bright_yellow(),
        "11-20".bright_black(),
        "███".bright_red()
    );
    println!("             {}", ">20 edges".bright_black());
}

fn print_embedding_projection(clusters: &[ClusterInfo]) {
    println!("  {}", "EMBEDDING SPACE PROJECTION".bright_blue().bold());
    println!(
        "  {}",
        "2D projection of cluster centroids (PCA-like reduction)"
            .bright_black()
            .italic()
    );
    println!();

    // Get clusters with centroids
    let clusters_with_centroids: Vec<_> = clusters
        .iter()
        .enumerate()
        .filter_map(|(idx, c)| c.centroid.as_ref().map(|cent| (idx, c, cent)))
        .collect();

    if clusters_with_centroids.is_empty() {
        println!("  {}", "No centroid data available".bright_black());
        return;
    }

    // Simple projection: use first two dimensions of centroid
    // (Real implementation would use PCA/t-SNE)
    let points: Vec<(usize, f32, f32)> = clusters_with_centroids
        .iter()
        .map(|(idx, _, cent)| {
            let x = if cent.len() > 0 { cent[0] } else { 0.0 };
            let y = if cent.len() > 1 { cent[1] } else { 0.0 };
            (*idx, x, y)
        })
        .collect();

    // Normalize to grid coordinates
    let min_x = points
        .iter()
        .map(|(_, x, _)| *x)
        .fold(f32::INFINITY, f32::min);
    let max_x = points
        .iter()
        .map(|(_, x, _)| *x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = points
        .iter()
        .map(|(_, _, y)| *y)
        .fold(f32::INFINITY, f32::min);
    let max_y = points
        .iter()
        .map(|(_, _, y)| *y)
        .fold(f32::NEG_INFINITY, f32::max);

    let width = 60;
    let height = 15;

    // Create grid
    let mut grid: Vec<Vec<Option<usize>>> = vec![vec![None; width]; height];

    for (idx, x, y) in &points {
        let norm_x = if max_x > min_x {
            ((x - min_x) / (max_x - min_x) * (width - 1) as f32) as usize
        } else {
            width / 2
        };
        let norm_y = if max_y > min_y {
            ((y - min_y) / (max_y - min_y) * (height - 1) as f32) as usize
        } else {
            height / 2
        };

        let norm_x = norm_x.min(width - 1);
        let norm_y = norm_y.min(height - 1);

        grid[norm_y][norm_x] = Some(*idx);
    }

    // Print grid
    println!(
        "  {}{}{}",
        TOP_LEFT,
        HORIZONTAL.repeat(width + 2),
        TOP_RIGHT
    );
    for row in grid.iter().rev() {
        print!("  {} ", VERTICAL);
        for cell in row {
            match cell {
                Some(idx) => {
                    // Color based on cluster size
                    let size = clusters[*idx].member_count;
                    let symbol = if size > 15 {
                        "◉".bright_red()
                    } else if size > 8 {
                        "●".bright_yellow()
                    } else {
                        "○".bright_green()
                    };
                    print!("{}", symbol);
                }
                None => print!("{}", "·".bright_black()),
            }
        }
        println!(" {}", VERTICAL);
    }
    println!(
        "  {}{}{}",
        BOTTOM_LEFT,
        HORIZONTAL.repeat(width + 2),
        BOTTOM_RIGHT
    );

    // Print legend
    println!();
    println!(
        "  {} {} {} {} {} {} {}",
        "Size:".bright_black(),
        "○".bright_green(),
        "1-8".bright_black(),
        "●".bright_yellow(),
        "9-15".bright_black(),
        "◉".bright_red(),
        ">15 nodes".bright_black()
    );
}
