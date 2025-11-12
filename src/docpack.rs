use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Metadata about a docpack file (matches backend expected format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    // Required fields
    pub name: String,
    pub version: String,
    pub ecosystem: String,

    // Optional fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
}

/// A single documentation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocEntry {
    // Identity
    pub id: String,
    #[serde(rename = "type")]
    pub entry_type: EntryType,
    pub name: String,
    pub path: String,

    // Content
    pub title: String,
    pub summary: String,
    pub content: String,

    // Discoverability
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aliases: Option<Vec<String>>,

    // Rich Content
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Example>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    // Metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EntryType {
    // Code constructs
    Function,
    Method,
    Struct,
    Class,
    Enum,
    Trait,
    Interface,
    Property,

    // Containers
    Module,
    Package,
    Namespace,

    // Documentation types
    Guide,
    Tutorial,
    Concept,
    Reference,

    // Special types
    CliCommand,
    ApiEndpoint,
    Error,
    Diagnostic,

    // Extensible
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    pub code: String,
    pub lang: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Manifest {
    /// Create a new manifest for a tool
    pub fn new(
        tool_name: impl Into<String>,
        tool_version: impl Into<String>,
        ecosystem: impl Into<String>,
    ) -> Self {
        let name = tool_name.into();
        let eco = ecosystem.into();
        Self {
            name: name.clone(),
            version: tool_version.into(),
            ecosystem: eco.clone(),
            summary: Some(format!("{} documentation", name)),
            description: None,
            homepage: None,
            tags: vec![eco],
            author: None,
            license: None,
        }
    }
}

impl DocEntry {
    /// Create a builder for constructing a DocEntry
    pub fn builder(
        id: impl Into<String>,
        entry_type: EntryType,
        name: impl Into<String>,
    ) -> DocEntryBuilder {
        DocEntryBuilder {
            id: id.into(),
            entry_type,
            name: name.into(),
            path: String::new(),
            title: String::new(),
            summary: String::new(),
            content: String::new(),
            tags: Vec::new(),
            aliases: None,
            examples: None,
            related: None,
            url: None,
            metadata: None,
        }
    }
}

/// Builder for DocEntry
pub struct DocEntryBuilder {
    id: String,
    entry_type: EntryType,
    name: String,
    path: String,
    title: String,
    summary: String,
    content: String,
    tags: Vec<String>,
    aliases: Option<Vec<String>>,
    examples: Option<Vec<Example>>,
    related: Option<Vec<String>>,
    url: Option<String>,
    metadata: Option<HashMap<String, serde_json::Value>>,
}

impl DocEntryBuilder {
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = path.into();
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.summary = summary.into();
        self
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    pub fn build(self) -> DocEntry {
        DocEntry {
            id: self.id,
            entry_type: self.entry_type,
            name: self.name,
            path: self.path,
            title: self.title,
            summary: self.summary,
            content: self.content,
            tags: self.tags,
            aliases: self.aliases,
            examples: self.examples,
            related: self.related,
            url: self.url,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = Manifest::new("test-tool", "1.0.0", "testing");
        assert_eq!(manifest.name, "test-tool");
        assert_eq!(manifest.version, "1.0.0");
        assert_eq!(manifest.ecosystem, "testing");
    }

    #[test]
    fn test_manifest_serialization() {
        let manifest = Manifest::new("test", "1.0.0", "testing");

        // Serialize to JSON
        let json = serde_json::to_string(&manifest);
        assert!(json.is_ok());

        // Deserialize back
        let json_str = json.unwrap();
        let deserialized: Result<Manifest, _> = serde_json::from_str(&json_str);
        assert!(deserialized.is_ok());

        let manifest2 = deserialized.unwrap();
        assert_eq!(manifest2.name, "test");
        assert_eq!(manifest2.version, "1.0.0");
        assert_eq!(manifest2.ecosystem, "testing");
    }

    #[test]
    fn test_doc_entry_builder() {
        let entry = DocEntry::builder("test::example", EntryType::Class, "Example")
            .path("test".to_string())
            .title("Example Class".to_string())
            .summary("An example".to_string())
            .content("# Example\n\nContent here".to_string())
            .tags(vec!["test".to_string()])
            .build();

        assert_eq!(entry.id, "test::example");
        assert_eq!(entry.name, "Example");
        assert_eq!(entry.path, "test");
    }

    #[test]
    fn test_entry_type_serialization() {
        let entry = DocEntry::builder("test::cmd", EntryType::CliCommand, "cmd")
            .path("test".to_string())
            .title("Command".to_string())
            .summary("A command".to_string())
            .content("Content".to_string())
            .tags(vec![])
            .build();

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains(r#""type":"cli-command""#));
    }
}
