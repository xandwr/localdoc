mod commands;
mod types;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "localdoc")]
#[command(about = "CLI tool for inspecting .docpack files", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List installed docpacks
    List,

    /// Generate a docpack from a source zip file or GitHub repository
    Generate {
        /// Path to source .zip file or GitHub repository URL
        #[arg(value_name = "INPUT")]
        input: String,
    },

    /// Show quick info about a docpack
    Info {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,
    },

    /// Show detailed statistics
    Stats {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,
    },

    /// List nodes in the graph
    Nodes {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Filter by node type (function, type, module, file, cluster)
        #[arg(short, long)]
        kind: Option<String>,

        /// Only show public nodes
        #[arg(short, long)]
        public: bool,

        /// Limit number of results
        #[arg(short, long, default_value = "50")]
        limit: usize,
    },

    /// Inspect a specific node by ID
    Inspect {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Node ID to inspect
        #[arg(value_name = "NODE_ID")]
        node_id: String,
    },

    /// Search for nodes by name
    Search {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Search query
        #[arg(value_name = "QUERY")]
        query: String,

        /// Case-sensitive search
        #[arg(short, long)]
        case_sensitive: bool,
    },

    /// Extract files from the docpack
    Extract {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Output directory
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Compare two docpacks
    Diff {
        /// Path to old .docpack file
        #[arg(value_name = "OLD")]
        old: PathBuf,

        /// Path to new .docpack file
        #[arg(value_name = "NEW")]
        new: PathBuf,
    },

    /// Show documentation for a node
    Explain {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Node ID to show documentation for
        #[arg(value_name = "NODE_ID")]
        node_id: String,
    },

    /// Visualize semantic subsystem clustering and architecture
    Map {
        /// Path to .docpack file
        #[arg(value_name = "FILE")]
        docpack: PathBuf,

        /// Compact output (less detail)
        #[arg(short, long)]
        compact: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => {
            commands::list::run()?;
        }
        Commands::Generate { input } => {
            commands::generate::run(input)?;
        }
        Commands::Info { docpack } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::info::run(resolved)?;
        }
        Commands::Stats { docpack } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::stats::run(resolved)?;
        }
        Commands::Nodes {
            docpack,
            kind,
            public,
            limit,
        } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::nodes::run(resolved, kind, public, limit)?;
        }
        Commands::Inspect { docpack, node_id } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::inspect::run(resolved, node_id)?;
        }
        Commands::Search {
            docpack,
            query,
            case_sensitive,
        } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::search::run(resolved, query, case_sensitive)?;
        }
        Commands::Extract { docpack, output } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::extract::run(resolved, output)?;
        }
        Commands::Diff { old, new } => {
            let old_resolved = commands::resolve_docpack_path(&old)?;
            let new_resolved = commands::resolve_docpack_path(&new)?;
            commands::diff::run(old_resolved, new_resolved)?;
        }
        Commands::Explain { docpack, node_id } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::explain::run(resolved, node_id)?;
        }
        Commands::Map { docpack, compact } => {
            let resolved = commands::resolve_docpack_path(&docpack)?;
            commands::map::run(resolved, compact)?;
        }
    }

    Ok(())
}
