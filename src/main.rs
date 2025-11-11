mod docpack;
mod godot_parser;
mod lister;
mod packer;
mod query;
mod telemetry;

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

        /// Maximum number of results to display
        #[arg(short, long, default_value = "10")]
        limit: usize,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Filter by entry type (e.g., "class", "method", "function", "property")
        #[arg(short = 't', long)]
        entry_type: Option<String>,

        /// Filter by path (e.g., "BaseMaterial3D" to only show results from that class)
        #[arg(short = 'p', long)]
        path: Option<String>,

        /// Filter by docpack name (e.g., "godot")
        #[arg(short = 'd', long)]
        docpack: Option<String>,

        /// Show verbose output with more context
        #[arg(short, long)]
        verbose: bool,
    },

    /// Show full documentation for a specific entry
    Show {
        /// Entry ID (e.g., "godot::BaseMaterial3D::stencil_mode")
        id: String,
    },

    /// Configure localdoc settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Set a configuration value
    Set {
        /// Configuration key (e.g., "telemetry")
        key: String,
        /// Configuration value (e.g., "true" or "false")
        value: String,
    },

    /// Get a configuration value
    Get {
        /// Configuration key (e.g., "telemetry")
        key: String,
    },

    /// Show all configuration
    Show,
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
                Some(Commands::Pack {
                    source,
                    output,
                    name,
                    version,
                    ecosystem: _,
                }) => {
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
                    let search_dirs = vec![PathBuf::from("."), localdoc_dir.clone()];

                    if let Err(e) = lister::list_docpacks(&search_dirs) {
                        eprintln!("❌ Error listing docpacks: {}", e);
                        std::process::exit(1);
                    }
                }
                Some(Commands::Query {
                    text,
                    limit,
                    json,
                    entry_type,
                    path,
                    docpack,
                    verbose,
                }) => {
                    if !json {
                        println!("🔍 Searching for: '{}'", text);
                    }

                    // Send telemetry event
                    telemetry::send_telemetry_event(
                        "query",
                        serde_json::json!({
                            "query_length": text.len(),
                            "has_filters": entry_type.is_some() || path.is_some() || docpack.is_some(),
                            "output_format": if json { "json" } else { "text" }
                        }),
                    );

                    // Search in current directory and localdoc directory
                    let search_dirs = vec![PathBuf::from("."), localdoc_dir.clone()];

                    // Create query options
                    let query_options = query::QueryOptions {
                        entry_type: entry_type.as_deref(),
                        path: path.as_deref(),
                        docpack: docpack.as_deref(),
                        verbose,
                    };

                    match query::query_docpacks(&search_dirs, &text, &query_options) {
                        Ok(results) => {
                            if json {
                                query::display_results_json(&results);
                            } else {
                                query::display_results(&results, Some(limit), verbose);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Error querying docpacks: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Some(Commands::Show { id }) => {
                    // Search in current directory and localdoc directory
                    let search_dirs = vec![PathBuf::from("."), localdoc_dir.clone()];

                    match query::show_entry(&search_dirs, &id) {
                        Ok(()) => {}
                        Err(e) => {
                            eprintln!("❌ Error showing entry: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Some(Commands::Config { action }) => match action {
                    ConfigAction::Set { key, value } => {
                        if key == "telemetry" {
                            let mut config = telemetry::load_config();
                            match value.to_lowercase().as_str() {
                                "true" | "on" | "yes" | "1" => {
                                    config.enabled = true;
                                    if let Err(e) = telemetry::save_config(&config) {
                                        eprintln!("❌ Error saving config: {}", e);
                                        std::process::exit(1);
                                    }
                                    println!("✅ Telemetry enabled");
                                    println!("📊 Anonymous usage data will be collected");
                                    #[cfg(feature = "telemetry")]
                                    println!("🆔 Client ID: {}", config.client_id);
                                }
                                "false" | "off" | "no" | "0" => {
                                    config.enabled = false;
                                    if let Err(e) = telemetry::save_config(&config) {
                                        eprintln!("❌ Error saving config: {}", e);
                                        std::process::exit(1);
                                    }
                                    println!("✅ Telemetry disabled");
                                    println!("🔒 No data will be collected");
                                }
                                _ => {
                                    eprintln!(
                                        "❌ Invalid value. Use: true/false, on/off, yes/no, or 1/0"
                                    );
                                    std::process::exit(1);
                                }
                            }
                        } else {
                            eprintln!("❌ Unknown config key: {}", key);
                            eprintln!("   Available keys: telemetry");
                            std::process::exit(1);
                        }
                    }
                    ConfigAction::Get { key } => {
                        if key == "telemetry" {
                            println!("{}", telemetry::telemetry_status());
                        } else {
                            eprintln!("❌ Unknown config key: {}", key);
                            eprintln!("   Available keys: telemetry");
                            std::process::exit(1);
                        }
                    }
                    ConfigAction::Show => {
                        println!("📋 localdoc Configuration\n");
                        println!("{}", telemetry::telemetry_status());
                        println!(
                            "\n📁 Config location: {}",
                            telemetry::get_config_path().display()
                        );
                    }
                },
                None => {
                    println!("📁 Using localdoc directory: {}", localdoc_dir.display());
                    println!("Welcome to localdoc! 👋");
                    println!("Run 'localdoc --help' for usage information.");

                    // Show telemetry status on first run
                    let config = telemetry::load_config();
                    if !config.enabled {
                        println!("\n{}", telemetry::telemetry_status());
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Error initializing localdoc: {}", e);
            std::process::exit(1);
        }
    }
}
