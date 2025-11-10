use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The docpack format version this implementation supports
pub const DOCPACK_VERSION: &str = "0.1.0";

/// Metadata about a docpack file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub docpack_version: String,
    pub tool: ToolInfo,
    pub metadata: DocpackMetadata,
    pub schema: SchemaInfo,
    #[serde(default)]
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    pub name: String,
    pub version: String,
    pub ecosystem: String,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocpackMetadata {
    pub generated_at: String,
    pub generator: String,
    pub content_hash: String,
    pub entry_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaInfo {
    pub version: String,
    #[serde(default)]
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
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
    pub fn new(tool_name: impl Into<String>, tool_version: impl Into<String>, ecosystem: impl Into<String>) -> Self {
        Self {
            docpack_version: DOCPACK_VERSION.to_string(),
            tool: ToolInfo {
                name: tool_name.into(),
                version: tool_version.into(),
                ecosystem: ecosystem.into(),
                homepage: None,
            },
            metadata: DocpackMetadata {
                generated_at: chrono::Utc::now().to_rfc3339(),
                generator: format!("localdoc-cli/{}", env!("CARGO_PKG_VERSION")),
                content_hash: String::new(),
                entry_count: 0,
            },
            schema: SchemaInfo {
                version: "1.0".to_string(),
                extensions: Vec::new(),
            },
            dependencies: Vec::new(),
        }
    }
}

impl DocEntry {
    /// Create a builder for constructing a DocEntry
    pub fn builder(id: impl Into<String>, entry_type: EntryType, name: impl Into<String>) -> DocEntryBuilder {
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

    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.url = Some(url.into());
        self
    }

    pub fn examples(mut self, examples: Vec<Example>) -> Self {
        self.examples = Some(examples);
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
