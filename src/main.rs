use clap::Parser;
use directories::UserDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "localdoc")]
#[command(author, version, about = "Local documentation manager", long_about = None)]
struct Cli {
    /// The command or query to run
    query: Option<String>,
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
            println!("📁 Using localdoc directory: {}", localdoc_dir.display());

            if let Some(query) = cli.query {
                println!("Running query: {}", query);
                // Add your actual logic here
            } else {
                println!("Welcome to localdoc! 👋");
                println!("Run 'localdoc --help' for usage information.");
            }
        }
        Err(e) => {
            eprintln!("❌ Error initializing localdoc: {}", e);
            std::process::exit(1);
        }
    }
}
