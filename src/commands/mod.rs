pub mod diff;
pub mod explain;
pub mod extract;
pub mod generate;
pub mod info;
pub mod inspect;
pub mod list;
pub mod map;
pub mod nodes;
pub mod search;
pub mod stats;

use crate::types::{DocpackGraph, Documentation, PackageMetadata};
use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

/// Get the default docpacks directory (~/.localdoc/docpacks/)
pub fn get_docpacks_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .context("Could not determine home directory")?;

    let docpacks_path = PathBuf::from(home).join(".localdoc").join("docpacks");

    Ok(docpacks_path)
}

/// Resolve a docpack path - if just a name is provided, look in ~/.localdoc/docpacks/
pub fn resolve_docpack_path(input: &Path) -> Result<PathBuf> {
    // If it's an absolute path or contains path separators, use as-is
    if input.is_absolute()
        || input.to_string_lossy().contains('/')
        || input.to_string_lossy().contains('\\')
    {
        return Ok(input.to_path_buf());
    }

    // Otherwise, treat it as a docpack name and look in the standard directory
    let docpacks_dir = get_docpacks_dir()?;
    let mut resolved = docpacks_dir.join(input);

    // Add .docpack extension if not present
    if resolved.extension().is_none() {
        resolved.set_extension("docpack");
    }

    Ok(resolved)
}

pub fn load_docpack(
    path: impl AsRef<Path>,
) -> Result<(DocpackGraph, PackageMetadata, Option<Documentation>)> {
    let file = std::fs::File::open(path.as_ref()).context("Failed to open .docpack file")?;

    let mut archive =
        zip::ZipArchive::new(file).context("Failed to read .docpack as zip archive")?;

    let mut graph_json = String::new();
    archive
        .by_name("graph.json")
        .context("graph.json not found in .docpack")?
        .read_to_string(&mut graph_json)?;

    let graph: DocpackGraph =
        serde_json::from_str(&graph_json).context("Failed to parse graph.json")?;

    let mut metadata_json = String::new();
    archive
        .by_name("metadata.json")
        .context("metadata.json not found in .docpack")?
        .read_to_string(&mut metadata_json)?;

    let metadata: PackageMetadata =
        serde_json::from_str(&metadata_json).context("Failed to parse metadata.json")?;

    let documentation = if let Ok(mut doc_file) = archive.by_name("documentation.json") {
        let mut doc_json = String::new();
        doc_file.read_to_string(&mut doc_json)?;
        match serde_json::from_str(&doc_json) {
            Ok(doc) => Some(doc),
            Err(e) => {
                eprintln!("Warning: Failed to parse documentation.json: {}", e);
                None
            }
        }
    } else {
        None
    };

    Ok((graph, metadata, documentation))
}
