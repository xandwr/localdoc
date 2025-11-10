mod docpack;
mod godot_parser;
mod lister;
mod packer;

use clap::{Parser, Subcommand};
use directories::UserDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "localdoc")]
#[command(author, version, about = "Local documentation manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Pack documentation from a directory into a .docpack file
    Pack {
        /// Source directory containing documentation
        #[arg(short, long)]
        source: PathBuf,
        
        /// Output .docpack file path
        #[arg(short, long)]
        output: PathBuf,
        
        /// Tool name
        #[arg(short, long)]
        name: String,
        
        /// Tool version
        #[arg(short, long)]
        version: String,
        
        /// Ecosystem (e.g., "game-engine", "programming-language")
        #[arg(short, long, default_value = "other")]
        ecosystem: String,
    },
    
    /// List all discovered docpacks
    List,
    
    /// Query installed docpacks
    Query {
        /// Search query
        text: String,
    },
}

fn get_localdoc_dir() -> PathBuf {
    if let Some(user_dirs) = UserDirs::new() {
        user_dirs.home_dir().join("localdoc")
    } else {
        // Fallback to current directory if we can't get home
        PathBuf::from("./localdoc")
    }
}

fn ensure_localdoc_initialized() -> Result<PathBuf, std::io::Error> {
    let localdoc_dir = get_localdoc_dir();

    if !localdoc_dir.exists() {
        println!(
            "🚀 First time setup: Creating localdoc directory at {}",
            localdoc_dir.display()
        );
        fs::create_dir_all(&localdoc_dir)?;
        println!("✅ localdoc initialized successfully!");
    }

    Ok(localdoc_dir)
}

fn main() {
    let cli = Cli::parse();

    // Lazy initialization - happens automatically on first run
    match ensure_localdoc_initialized() {
        Ok(localdoc_dir) => {
            match cli.command {
                Some(Commands::Pack { source, output, name, version, ecosystem: _ }) => {
                    println!("📦 Packing documentation from: {}", source.display());
                    println!("   Output: {}", output.display());
                    println!("   Tool: {} v{}", name, version);
                    
                    // Currently only supports Godot XML format
                    match packer::pack_godot_docs(&source, &output, &name, &version) {
                        Ok(()) => {
                            println!("\n🎉 Docpack created successfully!");
                        }
                        Err(e) => {
                            eprintln!("\n❌ Error creating docpack: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Some(Commands::List) => {
                    // Search in current directory and localdoc directory
                    let search_dirs = vec![
                        PathBuf::from("."),
                        localdoc_dir.clone(),
                    ];
                    
                    if let Err(e) = lister::list_docpacks(&search_dirs) {
                        eprintln!("❌ Error listing docpacks: {}", e);
                        std::process::exit(1);
                    }
                }
                Some(Commands::Query { text }) => {
                    println!("🔍 Searching for: {}", text);
                    // TODO: Implement query logic
                    println!("⚠️  Query not yet implemented");
                }
                None => {
                    println!("📁 Using localdoc directory: {}", localdoc_dir.display());
                    println!("Welcome to localdoc! 👋");
                    println!("Run 'localdoc --help' for usage information.");
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error initializing localdoc: {}", e);
            std::process::exit(1);
        }
    }
}
