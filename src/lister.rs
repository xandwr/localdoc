use serde_json;
use std::fs;
use std::path::{Path, PathBuf};

use crate::docpack::Manifest;

pub struct DocpackInfo {
    pub path: PathBuf,
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub entry_count: Option<usize>,
}

/// Discover docpacks in a directory
pub fn discover_docpacks(
    search_dir: &Path,
) -> Result<Vec<DocpackInfo>, Box<dyn std::error::Error>> {
    let mut docpacks = Vec::new();

    if !search_dir.exists() {
        return Ok(docpacks);
    }

    // Look for directories that contain manifest.json
    for entry in fs::read_dir(search_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_path = path.join("manifest.json");
            if manifest_path.exists() {
                match load_docpack_info(&path) {
                    Ok(info) => docpacks.push(info),
                    Err(e) => {
                        eprintln!("⚠️  Warning: Failed to read {}: {}", path.display(), e);
                    }
                }
            }
        }
    }

    Ok(docpacks)
}

/// Load docpack info from a directory
fn load_docpack_info(docpack_dir: &Path) -> Result<DocpackInfo, Box<dyn std::error::Error>> {
    let manifest_path = docpack_dir.join("manifest.json");
    let manifest_content = fs::read_to_string(manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_content)?;

    // Count entries from content.jsonl if it exists
    let content_path = docpack_dir.join("content.jsonl");
    let entry_count = if content_path.exists() {
        fs::read_to_string(&content_path)
            .ok()
            .map(|content| content.lines().count())
    } else {
        None
    };

    Ok(DocpackInfo {
        path: docpack_dir.to_path_buf(),
        name: manifest.name,
        version: manifest.version,
        ecosystem: manifest.ecosystem,
        entry_count,
    })
}

/// List all discovered docpacks with nice formatting
pub fn list_docpacks(search_dirs: &[PathBuf]) -> Result<(), Box<dyn std::error::Error>> {
    let mut all_docpacks = Vec::new();

    for dir in search_dirs {
        let mut docpacks = discover_docpacks(dir)?;
        all_docpacks.append(&mut docpacks);
    }

    if all_docpacks.is_empty() {
        println!("📭 No docpacks found.");
        println!("\nCreate one with:");
        println!(
            "  localdoc pack --source <dir> --output <name>.docpack --name <tool> --version <ver>"
        );
        return Ok(());
    }

    println!("📚 Discovered {} docpack(s):\n", all_docpacks.len());

    // Find max widths for formatting
    let max_name_width = all_docpacks
        .iter()
        .map(|d| d.name.len())
        .max()
        .unwrap_or(10);
    let max_version_width = all_docpacks
        .iter()
        .map(|d| d.version.len())
        .max()
        .unwrap_or(7);
    let max_ecosystem_width = all_docpacks
        .iter()
        .map(|d| d.ecosystem.len())
        .max()
        .unwrap_or(9);

    // Print header
    println!(
        "{:<width_name$}  {:<width_ver$}  {:<width_eco$}  {:>10}  {}",
        "NAME",
        "VERSION",
        "ECOSYSTEM",
        "ENTRIES",
        "LOCATION",
        width_name = max_name_width,
        width_ver = max_version_width,
        width_eco = max_ecosystem_width,
    );
    println!("{}", "─".repeat(80));

    // Print each docpack
    for docpack in &all_docpacks {
        println!(
            "{:<width_name$}  {:<width_ver$}  {:<width_eco$}  {:>10}  {}",
            docpack.name,
            docpack.version,
            docpack.ecosystem,
            format_number(docpack.entry_count),
            docpack.path.display(),
            width_name = max_name_width,
            width_ver = max_version_width,
            width_eco = max_ecosystem_width,
        );
    }

    println!("\n💡 Query a docpack with:");
    println!("  localdoc query <search-term>");

    Ok(())
}

fn format_number(n: Option<usize>) -> String {
    match n {
        Some(num) => {
            let s = num.to_string();
            let mut result = String::new();
            let mut count = 0;

            for c in s.chars().rev() {
                if count > 0 && count % 3 == 0 {
                    result.push(',');
                }
                result.push(c);
                count += 1;
            }

            result.chars().rev().collect()
        }
        None => "?".to_string(),
    }
}
