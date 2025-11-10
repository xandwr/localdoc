# The .docpack Format Specification v0.1

## Vision
The `.docpack` format is the universal standard for packaging, distributing, and querying tool documentation locally. It's the Rosetta Stone that bridges ecosystems—from Python to Rust, from CLI tools to web frameworks.

## Core Principles

1. **Minimal**: Simple structure, no bloat
2. **Consistent**: Same format across all tools and ecosystems
3. **Queryable**: Optimized for local search and retrieval
4. **Serializable**: Works as SQLite, JSONL, or binary
5. **Composable**: Docpacks can reference and extend each other

## Format Structure

### Container Format
A `.docpack` file is a structured archive containing:
```
tool-name.docpack
├── manifest.json          # Metadata and schema version
├── content.jsonl          # Line-delimited documentation entries
└── index.db              # Optional: Pre-built SQLite index
```

### Manifest Schema
```json
{
  "docpack_version": "0.1.0",
  "tool": {
    "name": "rust",
    "version": "1.70.0",
    "ecosystem": "programming-language",
    "homepage": "https://rust-lang.org"
  },
  "metadata": {
    "generated_at": "2025-11-09T00:00:00Z",
    "generator": "localdoc-cli",
    "content_hash": "sha256:...",
    "entry_count": 15234
  },
  "schema": {
    "version": "1.0",
    "extensions": []
  }
}
```

### Content Schema (JSONL)
Each line is a complete, queryable documentation entry:

```jsonl
{"id":"rust::std::vec::Vec","type":"struct","name":"Vec","path":"std::vec","title":"Vec<T, A>","summary":"A contiguous growable array type","content":"A contiguous growable array type, written as `Vec<T>`...","tags":["collection","dynamic","heap"],"url":"https://doc.rust-lang.org/std/vec/struct.Vec.html","examples":[{"code":"let mut v = Vec::new();\nv.push(1);","lang":"rust"}],"metadata":{"stability":"stable","since":"1.0.0"}}
{"id":"rust::std::string::String","type":"struct","name":"String","path":"std::string","title":"String","summary":"A UTF-8 encoded, growable string","content":"A UTF-8 encoded, growable string...","tags":["string","text","utf8"],"url":"https://doc.rust-lang.org/std/string/struct.String.html","examples":[{"code":"let s = String::from(\"hello\");","lang":"rust"}],"metadata":{"stability":"stable","since":"1.0.0"}}
```

### Entry Schema
```typescript
interface DocEntry {
  // Identity
  id: string;              // Unique identifier: "tool::module::item"
  type: EntryType;         // "function" | "struct" | "class" | "module" | "guide" | etc.
  name: string;            // Short name
  path: string;            // Hierarchical path (e.g., "std::collections")
  
  // Content
  title: string;           // Display title
  summary: string;         // One-line description (for search results)
  content: string;         // Full markdown documentation
  
  // Discoverability
  tags: string[];          // Searchable tags
  aliases?: string[];      // Alternative names
  
  // Rich Content
  examples?: Example[];    // Code examples
  related?: string[];      // IDs of related entries
  url?: string;            // Canonical online URL
  
  // Metadata
  metadata?: Record<string, any>;  // Tool-specific metadata
}

interface Example {
  code: string;
  lang: string;
  description?: string;
}

type EntryType = 
  | "function" | "method" | "struct" | "class" | "enum" | "trait" | "interface"
  | "module" | "package" | "namespace"
  | "guide" | "tutorial" | "concept" | "reference"
  | "cli-command" | "api-endpoint"
  | "error" | "diagnostic";
```

## Storage Backends

### 1. JSONL (Primary)
- Human-readable
- Streamable
- Easy to generate and parse
- One entry per line

### 2. SQLite (Query-Optimized)
```sql
CREATE TABLE entries (
  id TEXT PRIMARY KEY,
  type TEXT NOT NULL,
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  title TEXT NOT NULL,
  summary TEXT NOT NULL,
  content TEXT NOT NULL,
  tags TEXT,  -- JSON array
  url TEXT,
  metadata TEXT,  -- JSON object
  examples TEXT  -- JSON array
);

CREATE INDEX idx_name ON entries(name);
CREATE INDEX idx_path ON entries(path);
CREATE INDEX idx_type ON entries(type);
CREATE VIRTUAL TABLE entries_fts USING fts5(name, title, summary, content, tags);
```

### 3. Binary (Future)
- Compressed format for large docpacks
- Uses MessagePack or similar

## Composition & References

Docpacks can reference other docpacks:

```json
{
  "dependencies": [
    {"name": "rust-std", "version": "1.70.0"},
    {"name": "tokio", "version": "1.32.0"}
  ]
}
```

## Distribution

### Registry (Future)
```bash
localdoc install rust
localdoc install django
localdoc update
```

### Manual Installation
```bash
localdoc add ./custom-tool.docpack
```

## Querying API

Local queries should support:
- Full-text search
- Fuzzy matching
- Type filtering
- Tag filtering
- Path-based lookup

```bash
localdoc query "vector push"
localdoc query "Vec::push" --exact
localdoc query --type function --tags async
```

## Ecosystem Examples

### Rust Docpack
```
rust-std.docpack
- std::vec::Vec
- std::string::String
- std::collections::HashMap
```

### Python Docpack
```
python-stdlib.docpack
- builtins::list
- builtins::dict
- pathlib::Path
```

### CLI Tool Docpack
```
git.docpack
- cli-command::git-commit
- cli-command::git-push
- concept::branches
```

## Versioning

Docpack spec follows semantic versioning:
- Major: Breaking changes to format
- Minor: Backward-compatible additions
- Patch: Clarifications, fixes

Current version: **0.1.0**

## License & Governance

The `.docpack` specification itself is public domain (CC0).
Implementations and specific docpack content follow their own licenses.

---

**This is the foundation. The standard. The protocol.**
