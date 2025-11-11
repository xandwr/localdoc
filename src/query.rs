use crate::docpack::{DocEntry, EntryType, Manifest};
use serde::Serialize;
use std::collections::HashSet;
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

pub struct QueryOptions<'a> {
    pub entry_type: Option<&'a str>,
    pub path: Option<&'a str>,
    pub docpack: Option<&'a str>,
    #[allow(dead_code)]
    pub verbose: bool,
}

/// Query docpacks with a fuzzy search string
pub fn query_docpacks(
    search_dirs: &[PathBuf],
    query: &str,
    options: &QueryOptions,
) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results = Vec::new();
    let query_lower = query.to_lowercase();

    // Find all .docpack directories or directories with manifest.json
    let docpack_dirs = find_docpack_dirs(search_dirs);

    // Collect all available terms for suggestions
    let mut all_terms = HashSet::new();

    // Search each docpack
    for docpack_dir in &docpack_dirs {
        if let Ok(docpack_results) =
            search_docpack(docpack_dir, &query_lower, options, &mut all_terms)
        {
            results.extend(docpack_results);
        }
    }

    // Sort by score (highest first)
    results.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // If no results, provide suggestions
    if results.is_empty() {
        suggest_alternatives(&query_lower, &all_terms);
    }

    Ok(results)
}

/// Find all docpack directories
fn find_docpack_dirs(search_dirs: &[PathBuf]) -> Vec<PathBuf> {
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
    docpack_dirs
}

/// Search a single docpack directory
fn search_docpack(
    docpack_dir: &Path,
    query: &str,
    options: &QueryOptions,
    all_terms: &mut HashSet<String>,
) -> Result<Vec<SearchResult>, std::io::Error> {
    let mut results = Vec::new();

    // Read manifest to get docpack name
    let manifest_path = docpack_dir.join("manifest.json");
    let manifest_file = File::open(&manifest_path)?;
    let manifest: Manifest = serde_json::from_reader(manifest_file)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let docpack_name = format!("{} v{}", manifest.tool.name, manifest.tool.version);

    // Check docpack filter
    if let Some(filter_docpack) = options.docpack {
        if !docpack_name
            .to_lowercase()
            .contains(&filter_docpack.to_lowercase())
        {
            return Ok(results);
        }
    }

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

        // Collect terms for suggestions
        all_terms.insert(entry.name.to_lowercase());
        for tag in &entry.tags {
            all_terms.insert(tag.to_lowercase());
        }

        // Apply filters
        if !matches_filters(&entry, &docpack_name, options) {
            continue;
        }

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

/// Check if an entry matches the filter options
fn matches_filters(entry: &DocEntry, _docpack_name: &str, options: &QueryOptions) -> bool {
    // Filter by entry type
    if let Some(filter_type) = options.entry_type {
        let filter_lower = filter_type.to_lowercase();
        let entry_type_str = match &entry.entry_type {
            EntryType::Class => "class",
            EntryType::Function => "function",
            EntryType::Method => "method",
            EntryType::Module => "module",
            EntryType::Struct => "struct",
            EntryType::Enum => "enum",
            EntryType::Trait => "trait",
            EntryType::Interface => "interface",
            EntryType::Package => "package",
            EntryType::Namespace => "namespace",
            EntryType::Guide => "guide",
            EntryType::Tutorial => "tutorial",
            EntryType::Concept => "concept",
            EntryType::Reference => "reference",
            EntryType::CliCommand => "command",
            EntryType::ApiEndpoint => "endpoint",
            EntryType::Error => "error",
            EntryType::Diagnostic => "diagnostic",
            EntryType::Other(s) => s.as_str(),
        };

        if !entry_type_str.to_lowercase().contains(&filter_lower) {
            return false;
        }
    }

    // Filter by path
    if let Some(filter_path) = options.path {
        if !entry
            .path
            .to_lowercase()
            .contains(&filter_path.to_lowercase())
        {
            return false;
        }
    }

    true
}

/// Calculate a fuzzy match score for an entry
/// Returns Some(score) if the entry matches, None otherwise
/// Higher scores are better matches
fn calculate_match_score(entry: &DocEntry, query: &str) -> Option<f32> {
    let mut score = 0.0f32;
    let query_lower = query.to_lowercase();

    // Split query into words for multi-word matching
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    let is_multi_word = query_words.len() > 1;

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
    // Multi-word: check if name contains all words
    else if is_multi_word {
        let name_lower = entry.name.to_lowercase();
        let matches = query_words
            .iter()
            .filter(|&word| name_lower.contains(word))
            .count();
        if matches > 0 {
            score += 30.0 * (matches as f32 / query_words.len() as f32);
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
    let content_lower = entry.content.to_lowercase();
    if content_lower.contains(&query_lower) {
        score += 5.0;
    } else if is_multi_word {
        // Multi-word content matching
        let matches = query_words
            .iter()
            .filter(|&word| content_lower.contains(word))
            .count();
        if matches > 0 {
            score += 3.0 * (matches as f32 / query_words.len() as f32);
        }
    }

    // Return score only if there's a match
    if score > 0.0 { Some(score) } else { None }
}

/// Suggest alternative search terms when no results are found
fn suggest_alternatives(query: &str, all_terms: &HashSet<String>) {
    println!("\n💡 Suggestions:");

    // Split query into words
    let query_words: Vec<&str> = query.split_whitespace().collect();

    // Find similar terms
    let mut suggestions: Vec<(String, usize)> = Vec::new();

    for term in all_terms {
        let mut similarity = 0;

        // Check if term contains any query word
        for word in &query_words {
            if term.contains(word) {
                similarity += 10;
            }
            // Check if term starts with query word
            if term.starts_with(word) {
                similarity += 5;
            }
            // Check for common substring (at least 3 chars)
            if word.len() >= 3 {
                for i in 0..=word.len() - 3 {
                    let substr = &word[i..i + 3];
                    if term.contains(substr) {
                        similarity += 1;
                    }
                }
            }
        }

        if similarity > 0 {
            suggestions.push((term.clone(), similarity));
        }
    }

    // Sort by similarity and take top 10
    suggestions.sort_by(|a, b| b.1.cmp(&a.1));
    suggestions.truncate(10);

    if suggestions.is_empty() {
        println!("   Try different search terms or check your spelling.");
        println!("   Use 'localdoc list' to see available docpacks.");
    } else {
        println!("   Did you mean:");
        for (term, _) in suggestions {
            println!("   • {}", term);
        }
    }
}

/// Display search results in a user-friendly format
pub fn display_results(results: &[SearchResult], limit: Option<usize>, verbose: bool) {
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

        if verbose {
            // Verbose mode: show more details
            println!("   ID: {}", entry.id);
            println!("   Summary: {}", entry.summary);

            // Show a longer snippet of content
            let content_preview = get_content_preview(&entry.content, 300);
            if !content_preview.is_empty() && content_preview != entry.summary {
                println!("   Content: {}", content_preview);
            }

            if let Some(url) = &entry.url {
                println!("   URL: {}", url);
            }

            if !entry.tags.is_empty() {
                println!("   Tags: {}", entry.tags.join(", "));
            }

            println!("   Score: {:.1}", result.score);
        } else {
            // Normal mode: concise output
            println!("   Summary: {}", truncate_string(&entry.summary, 100));

            // Show a snippet of the actual content (excluding markdown headers)
            let content_preview = get_content_preview(&entry.content, 150);
            if !content_preview.is_empty() && content_preview != entry.summary {
                println!("   Content: {}", content_preview);
            }

            if !entry.tags.is_empty() {
                println!("   Tags: {}", entry.tags.join(", "));
            }
        }

        println!("\n   💡 View full docs: localdoc show \"{}\"", entry.id);
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

/// Extract a preview of content, skipping markdown headers and empty lines
fn get_content_preview(content: &str, max_len: usize) -> String {
    let mut preview = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        // Skip markdown headers, empty lines, and type declarations
        if trimmed.is_empty()
            || trimmed.starts_with('#')
            || trimmed.starts_with("**Type:**")
            || trimmed.starts_with("```")
        {
            continue;
        }

        if !preview.is_empty() {
            preview.push(' ');
        }
        preview.push_str(trimmed);

        if preview.len() >= max_len {
            break;
        }
    }

    truncate_string(&preview, max_len)
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

/// Show full documentation for a specific entry by ID
pub fn show_entry(search_dirs: &[PathBuf], entry_id: &str) -> Result<(), std::io::Error> {
    // Find all docpacks
    let docpack_dirs = find_docpack_dirs(search_dirs);

    // Search for the entry
    for docpack_dir in docpack_dirs {
        // Read manifest
        let manifest_path = docpack_dir.join("manifest.json");
        let manifest_file = File::open(&manifest_path)?;
        let manifest: Manifest = serde_json::from_reader(manifest_file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let docpack_name = format!("{} v{}", manifest.tool.name, manifest.tool.version);

        // Search content.jsonl
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

            if entry.id == entry_id {
                // Found it! Display full documentation
                println!("\n📖 {}", entry.title);
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("\n📦 Docpack: {}", docpack_name);
                println!("🏷️  Type: {:?}", entry.entry_type);
                println!("📍 Path: {}", entry.path);
                println!("🆔 ID: {}", entry.id);

                if let Some(url) = &entry.url {
                    println!("🔗 URL: {}", url);
                }

                if !entry.tags.is_empty() {
                    println!("🏷️  Tags: {}", entry.tags.join(", "));
                }

                println!("\n{}\n", entry.content);

                return Ok(());
            }
        }
    }

    println!("❌ Entry not found: {}", entry_id);
    println!("\n💡 Tip: Use 'localdoc query <search>' to find entries and their IDs.");

    Ok(())
}
