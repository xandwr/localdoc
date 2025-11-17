use crate::models::{Documentation, Manifest, Symbol};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use zip::ZipArchive;

pub struct Docpack {
    pub manifest: Manifest,
    pub symbols: Vec<Symbol>,
    docs_cache: HashMap<String, Documentation>,
    archive: ZipArchive<File>,
}

impl Docpack {
    pub fn open(path: &str) -> Result<Self> {
        let file = File::open(path).context("Failed to open docpack file")?;
        let mut archive = ZipArchive::new(file).context("Failed to read docpack as ZIP archive")?;

        // Read manifest
        let manifest = {
            let mut manifest_file = archive
                .by_name("manifest.json")
                .context("manifest.json not found in docpack")?;
            let mut content = String::new();
            manifest_file.read_to_string(&mut content)?;
            serde_json::from_str(&content).context("Failed to parse manifest.json")?
        };

        // Read symbols
        let symbols = {
            let mut symbols_file = archive
                .by_name("symbols.json")
                .context("symbols.json not found in docpack")?;
            let mut content = String::new();
            symbols_file.read_to_string(&mut content)?;
            serde_json::from_str(&content).context("Failed to parse symbols.json")?
        };

        Ok(Docpack {
            manifest,
            symbols,
            docs_cache: HashMap::new(),
            archive,
        })
    }

    pub fn get_documentation(&mut self, doc_id: &str) -> Result<Documentation> {
        if let Some(doc) = self.docs_cache.get(doc_id) {
            return Ok(doc.clone());
        }

        let doc_path = format!("docs/{}.json", doc_id);
        let mut doc_file = self
            .archive
            .by_name(&doc_path)
            .context(format!("Documentation file {} not found", doc_path))?;

        let mut content = String::new();
        doc_file.read_to_string(&mut content)?;
        let doc: Documentation =
            serde_json::from_str(&content).context(format!("Failed to parse {}", doc_path))?;

        self.docs_cache.insert(doc_id.to_string(), doc.clone());
        Ok(doc)
    }

    pub fn find_symbols_by_name(&self, name: &str) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.id.contains(name))
            .collect()
    }

    pub fn find_symbols_by_file(&self, file: &str) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|s| s.file.contains(file))
            .collect()
    }

    pub fn search_symbols(&mut self, keyword: &str) -> Result<Vec<(Symbol, Documentation)>> {
        let keyword_lower = keyword.to_lowercase();
        let mut results = Vec::new();

        // Clone symbols to avoid borrow checker issues
        let symbols = self.symbols.clone();

        for symbol in &symbols {
            let doc = self.get_documentation(&symbol.doc_id)?;

            if symbol.id.to_lowercase().contains(&keyword_lower)
                || symbol.signature.to_lowercase().contains(&keyword_lower)
                || doc.summary.to_lowercase().contains(&keyword_lower)
                || doc.description.to_lowercase().contains(&keyword_lower)
            {
                results.push((symbol.clone(), doc));
            }
        }

        Ok(results)
    }

    pub fn get_unique_files(&self) -> Vec<String> {
        let mut files: Vec<String> = self.symbols.iter().map(|s| s.file.clone()).collect();
        files.sort();
        files.dedup();
        files
    }
}
