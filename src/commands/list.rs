use anyhow::Result;
use colored::Colorize;
use std::fs;

/// List all installed docpacks in ~/.localdoc/docpacks/
pub fn run() -> Result<()> {
    let docpacks_dir = super::get_docpacks_dir()?;

    if !docpacks_dir.exists() {
        println!("\n{}", "No docpacks directory found.".bright_yellow());
        println!(
            "{}",
            format!("Expected location: {:?}", docpacks_dir).bright_black()
        );
        println!("{}", "Run the builder to create docpacks.".bright_black());
        return Ok(());
    }

    let entries = fs::read_dir(&docpacks_dir)?;
    let mut docpacks: Vec<_> = entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().and_then(|s| s.to_str()) == Some("docpack"))
        .collect();

    if docpacks.is_empty() {
        println!("\n{}", "No docpacks found.".bright_yellow());
        println!("{}", format!("Location: {:?}", docpacks_dir).bright_black());
        println!("{}", "Run the builder to create docpacks.".bright_black());
        return Ok(());
    }

    docpacks.sort_by_key(|e| e.file_name());

    println!(
        "\n{}",
        format!("Installed Docpacks ({} total)", docpacks.len())
            .bright_cyan()
            .bold()
    );
    println!("{}", format!("Location: {:?}", docpacks_dir).bright_black());
    println!("{}", "=".repeat(80).bright_black());

    for entry in docpacks {
        let path = entry.path();
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        let metadata = entry.metadata()?;
        let size_kb = metadata.len() as f64 / 1024.0;

        // Try to get modified time
        let modified = metadata.modified().ok().map(|time| {
            use std::time::UNIX_EPOCH;
            if let Ok(duration) = time.duration_since(UNIX_EPOCH) {
                let secs = duration.as_secs();
                let days = secs / 86400;

                if days == 0 {
                    "Today".to_string()
                } else if days == 1 {
                    "Yesterday".to_string()
                } else if days < 7 {
                    format!("{} days ago", days)
                } else if days < 30 {
                    format!("{} weeks ago", days / 7)
                } else {
                    format!("{} months ago", days / 30)
                }
            } else {
                "Unknown".to_string()
            }
        });

        println!(
            "{} {}",
            name.bright_white().bold(),
            format!("({:.2} KB)", size_kb).bright_black()
        );

        if let Some(mod_time) = modified {
            println!(
                "       {}",
                format!("Modified: {}", mod_time).bright_black()
            );
        }
    }

    println!(
        "\n{}",
        "Use 'localdoc info <name>' to inspect a docpack".bright_black()
    );
    println!();

    Ok(())
}
