use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use walkdir::WalkDir;

use crate::docpack::{DocEntry, Manifest};
use crate::godot_parser;

pub fn pack_godot_docs(
    source_dir: &Path,
    output_path: &Path,
    name: &str,
    version: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Scanning {} for XML files...", source_dir.display());
    
    let mut all_entries = Vec::new();
    let mut file_count = 0;
    
    // Walk through all XML files
    for entry in WalkDir::new(source_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("xml") {
            file_count += 1;
            print!("\r   Parsing file {} ...", file_count);
            std::io::stdout().flush()?;
            
            match godot_parser::parse_godot_xml(path) {
                Ok(mut entries) => {
                    all_entries.append(&mut entries);
                }
                Err(e) => {
                    eprintln!("\n⚠️  Warning: Failed to parse {}: {}", path.display(), e);
                }
            }
        }
    }
    
    println!("\n✅ Parsed {} files, generated {} documentation entries", file_count, all_entries.len());
    
    // Create output directory structure
    let output_dir = output_path.parent().unwrap_or(Path::new("."));
    fs::create_dir_all(output_dir)?;
    
    // Remove existing output if it exists
    if output_path.exists() {
        println!("   Removing existing docpack...");
        if output_path.is_dir() {
            fs::remove_dir_all(output_path)?;
        } else {
            fs::remove_file(output_path)?;
        }
    }
    
    // Create the docpack directory
    fs::create_dir_all(output_path)?;
    
    // Create manifest
    println!("📝 Creating manifest...");
    let mut manifest = godot_parser::create_godot_manifest(version);
    manifest.metadata.entry_count = all_entries.len();
    manifest.metadata.content_hash = format!("sha256:placeholder"); // TODO: Calculate actual hash
    
    let manifest_path = output_path.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;
    
    // Write JSONL content
    println!("💾 Writing {} entries to JSONL...", all_entries.len());
    let content_path = output_path.join("content.jsonl");
    let file = File::create(&content_path)?;
    let mut writer = BufWriter::new(file);
    
    for entry in &all_entries {
        let json_line = serde_json::to_string(&entry)?;
        writeln!(writer, "{}", json_line)?;
    }
    
    writer.flush()?;
    
    println!("✅ Successfully created docpack!");
    println!("   Location: {}", output_path.display());
    println!("   Entries: {}", all_entries.len());
    println!("   Files: manifest.json, content.jsonl");
    
    Ok(())
}
