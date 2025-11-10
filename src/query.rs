use crate::docpack::{DocEntry, Manifest};
use serde::Serialize;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub entry: DocEntry,
    pub docpack_name: String,
    pub score: f32,
}

/// Query docpacks with a fuzzy search string
pub fn query_docpacks(search_dirs: &[PathBuf], query: &str) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();
    
    // Find all .docpack directories or directories with manifest.json
    let mut docpack_dirs = Vec::new();
    for dir in search_dirs {
        if !dir.exists() {
            continue;
        }
        
        for entry in WalkDir::new(dir)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_dir() {
                // Check if it's a .docpack directory or has a manifest.json file
                let is_docpack = path.extension().and_then(|s| s.to_str()) == Some("docpack");
                let has_manifest = path.join("manifest.json").exists();
                
                if is_docpack || has_manifest {
                    docpack_dirs.push(path.to_path_buf());
                }
            }
        }
    }
    
    // Search each docpack
    for docpack_dir in docpack_dirs {
        if let Ok(docpack_results) = search_docpack(&docpack_dir, &query_lower) {
            results.extend(docpack_results);
        }
    }
    
    // Sort by score (highest first)
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(results)
}

/// Search a single docpack directory
fn search_docpack(docpack_dir: &Path, query: &str) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results = Vec::new();
    
    // Read manifest to get docpack name
    let manifest_path = docpack_dir.join("manifest.json");
    let manifest_file = File::open(&manifest_path)?;
    let manifest: Manifest = serde_json::from_reader(manifest_file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    
    let docpack_name = format!("{} v{}", manifest.tool.name, manifest.tool.version);
    
    // Read content.jsonl and search each entry
    let content_path = docpack_dir.join("content.jsonl");
    let content_file = File::open(&content_path)?;
    let reader = BufReader::new(content_file);
    
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        
        let entry: DocEntry = serde_json::from_str(&line)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        
        // Calculate match score
        if let Some(score) = calculate_match_score(&entry, query) {
            results.push(SearchResult {
                entry,
                docpack_name: docpack_name.clone(),
                score,
            });
        }
    }
    
    Ok(results)
}

/// Calculate a fuzzy match score for an entry
/// Returns Some(score) if the entry matches, None otherwise
/// Higher scores are better matches
fn calculate_match_score(entry: &DocEntry, query: &str) -> Option<f32> {
    let mut score = 0.0f32;
    let query_lower = query.to_lowercase();
    
    // Exact name match (highest priority)
    if entry.name.to_lowercase() == query_lower {
        score += 100.0;
    }
    // Name contains query
    else if entry.name.to_lowercase().contains(&query_lower) {
        score += 50.0;
        // Bonus if it starts with the query
        if entry.name.to_lowercase().starts_with(&query_lower) {
            score += 25.0;
        }
    }
    
    // Check aliases
    if let Some(aliases) = &entry.aliases {
        for alias in aliases {
            if alias.to_lowercase() == query_lower {
                score += 90.0;
            } else if alias.to_lowercase().contains(&query_lower) {
                score += 40.0;
            }
        }
    }
    
    // Title match
    if entry.title.to_lowercase().contains(&query_lower) {
        score += 30.0;
    }
    
    // Path match (useful for namespaced items)
    if entry.path.to_lowercase().contains(&query_lower) {
        score += 20.0;
    }
    
    // Summary match
    if entry.summary.to_lowercase().contains(&query_lower) {
        score += 15.0;
    }
    
    // Tag match
    for tag in &entry.tags {
        if tag.to_lowercase() == query_lower {
            score += 25.0;
        } else if tag.to_lowercase().contains(&query_lower) {
            score += 10.0;
        }
    }
    
    // Content match (lower priority)
    if entry.content.to_lowercase().contains(&query_lower) {
        score += 5.0;
    }
    
    // Return score only if there's a match
    if score > 0.0 {
        Some(score)
    } else {
        None
    }
}

/// Display search results in a user-friendly format
pub fn display_results(results: &[SearchResult], limit: Option<usize>) {
    let results_to_show = if let Some(limit) = limit {
        &results[..results.len().min(limit)]
    } else {
        results
    };
    
    if results_to_show.is_empty() {
        println!("No results found.");
        return;
    }
    
    println!("\n📚 Found {} result(s):\n", results.len());
    
    for (i, result) in results_to_show.iter().enumerate() {
        let entry = &result.entry;
        
        println!("{}. {} [{}]", i + 1, entry.name, result.docpack_name);
        println!("   Type: {:?}", entry.entry_type);
        println!("   Path: {}", entry.path);
        println!("   Summary: {}", truncate_string(&entry.summary, 100));
        
        if let Some(url) = &entry.url {
            println!("   URL: {}", url);
        }
        
        if !entry.tags.is_empty() {
            println!("   Tags: {}", entry.tags.join(", "));
        }
        
        println!();
    }
    
    if let Some(limit) = limit {
        if results.len() > limit {
            println!("... and {} more result(s)", results.len() - limit);
        }
    }
}

/// Truncate a string to a maximum length, adding "..." if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

/// Display search results as JSON
pub fn display_results_json(results: &[SearchResult]) {
    match serde_json::to_string_pretty(&results) {
        Ok(json) => println!("{}", json),
        Err(e) => {
            eprintln!("❌ Error serializing results to JSON: {}", e);
            std::process::exit(1);
        }
    }
}
