use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn run(docpack: PathBuf, output_dir: PathBuf) -> Result<()> {
    let file = std::fs::File::open(&docpack).context("Failed to open .docpack file")?;

    let mut archive =
        zip::ZipArchive::new(file).context("Failed to read .docpack as zip archive")?;

    if !output_dir.exists() {
        std::fs::create_dir_all(&output_dir)?;
        println!(
            "{}",
            format!("Created directory: {}", output_dir.display()).bright_green()
        );
    }

    println!("\n{}", "Extracting files...".bright_cyan().bold());
    println!("{}", "=".repeat(50).bright_black());

    let mut extracted = 0;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = output_dir.join(file.name());

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }

            let mut outfile = std::fs::File::create(&outpath)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            outfile.write_all(&buffer)?;

            let size = buffer.len();
            println!(
                "  {} {} ({} bytes)",
                "✓".bright_green(),
                file.name().bright_white(),
                size.to_string().bright_black()
            );

            extracted += 1;
        }
    }

    println!(
        "\n{}",
        format!("Extracted {} files to {}", extracted, output_dir.display()).bright_green()
    );
    println!();

    Ok(())
}
