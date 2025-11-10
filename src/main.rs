use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "localdoc")]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new localdoc project
    Init {
        /// Optional path to initialize (defaults to current directory)
        path: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { path } => {
            let target_path = path.as_deref().unwrap_or(".");
            println!("Initializing localdoc in: {}", target_path);
            // Add initialization logic here
        }
    }
}