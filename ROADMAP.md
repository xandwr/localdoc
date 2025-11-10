# Technical Roadmap

## Core Components

### 1. Format Specification ✅
- [x] Define manifest schema
- [x] Define entry schema
- [x] Define storage backends (JSONL, SQLite)
- [x] Document composition model
- [ ] Finalize v1.0 spec

### 2. Core Library (`localdoc-core`)
```
src/
├── lib.rs              # Public API
├── docpack.rs          # Types & schemas ✅
├── manifest.rs         # Manifest parsing
├── storage/
│   ├── mod.rs
│   ├── jsonl.rs        # JSONL backend
│   ├── sqlite.rs       # SQLite backend
│   └── memory.rs       # In-memory for testing
├── query/
│   ├── mod.rs
│   ├── parser.rs       # Query DSL
│   ├── fts.rs          # Full-text search
│   └── index.rs        # Indexing engine
└── generator/
    ├── mod.rs
    └── builder.rs      # Builder API for creating docpacks
```

**Priority tasks**:
- [ ] JSONL reader/writer
- [ ] SQLite backend with FTS5
- [ ] Query parser and executor
- [ ] Docpack builder API

### 3. CLI (`localdoc`)
```
Commands:
├── localdoc                    # Status, welcome
├── localdoc query <text>       # Search across all docpacks
├── localdoc install <name>     # Download and install docpack
├── localdoc list               # List installed docpacks
├── localdoc info <name>        # Show docpack details
├── localdoc update             # Update all docpacks
├── localdoc remove <name>      # Uninstall docpack
├── localdoc pack <dir>         # Create docpack from directory
└── localdoc serve              # Local web UI (future)
```

**Priority tasks**:
- [x] Auto-initialization
- [ ] Query command with FTS
- [ ] List/info commands
- [ ] Pack command (docpack generator)

### 4. Generators

#### 4.1 Rust Documentation Generator
```bash
# From rustdoc JSON
localdoc-gen-rust --input target/doc/ --output rust-mylib.docpack

# From cargo metadata
cargo localdoc
```

#### 4.2 Python Documentation Generator
```bash
# From Sphinx build
localdoc-gen-python --sphinx _build/html --output python-mylib.docpack

# From pydoc
python -m localdoc.gen mypackage
```

#### 4.3 CLI Documentation Generator
```bash
# From --help output
localdoc-gen-cli --command git --output git.docpack

# From man pages
localdoc-gen-man --pages /usr/share/man --output unix-tools.docpack
```

**Priority tasks**:
- [ ] Rust generator (rustdoc JSON → docpack)
- [ ] Generic CLI help parser
- [ ] Python Sphinx converter

### 5. Registry & Distribution

**Phase 1: Static Registry**
```
https://docpacks.dev/
├── index.json              # List of all docpacks
└── packs/
    ├── rust-std-1.70.0.docpack
    ├── python-stdlib-3.11.0.docpack
    └── ...
```

**Phase 2: Dynamic Registry (API)**
```
POST   /api/v1/publish         # Publish a docpack
GET    /api/v1/search          # Search registry
GET    /api/v1/download/:name  # Download docpack
GET    /api/v1/metadata/:name  # Get metadata
```

**Priority tasks**:
- [ ] Static file hosting
- [ ] Registry API design
- [ ] Authentication for publishing
- [ ] CDN setup

### 6. IDE Integrations

#### VS Code Extension
```typescript
// Inline documentation on hover
// Quick search widget (Cmd+Shift+D)
// Context-aware suggestions
```

#### Language Server Protocol (LSP)
```
Add docpack support to language servers:
- rust-analyzer
- pyright
- typescript-language-server
```

**Priority tasks**:
- [ ] VS Code extension POC
- [ ] LSP protocol additions

## Development Phases

### Phase 1: MVP (Weeks 1-4) 🎯
**Goal**: Basic working CLI with local search

- [ ] Core types and schemas
- [ ] JSONL storage backend
- [ ] Basic query (string matching)
- [ ] CLI: query, list, info
- [ ] Create 3 example docpacks manually
- [ ] Published README + SPEC

**Deliverable**: Can run `localdoc query "Vec::push"` and get results

### Phase 2: Real Usage (Weeks 5-8)
**Goal**: Generate docpacks from real sources

- [ ] Rust generator (rustdoc → docpack)
- [ ] CLI help generator
- [ ] SQLite backend with FTS
- [ ] Better query parsing
- [ ] CLI: install, update, remove (from local files)
- [ ] Generate docpacks for: Rust std, Cargo, Git, ripgrep

**Deliverable**: Can generate and query real documentation

### Phase 3: Distribution (Weeks 9-12)
**Goal**: Public registry and ecosystem

- [ ] Static docpack registry
- [ ] CLI: install from registry
- [ ] Python generator
- [ ] 50+ published docpacks
- [ ] Basic web frontend for browsing
- [ ] Documentation and guides

**Deliverable**: Anyone can install and use docpacks

### Phase 4: Integration (Weeks 13-16)
**Goal**: Become useful in real workflows

- [ ] VS Code extension
- [ ] AI assistant integration (context provider)
- [ ] Advanced query features (filters, ranking)
- [ ] Performance optimizations
- [ ] Community contributions

**Deliverable**: Developers use it daily

## Technical Decisions

### Storage: Why JSONL?
- **Human readable**: Can inspect with `less`, `grep`, etc.
- **Streamable**: Don't need to load entire file
- **Line-based**: Easy to append, easy to parse
- **Universal**: Works everywhere, no special tools

### Storage: Why SQLite?
- **Performance**: FTS5 is excellent for full-text search
- **Portable**: Single file, no server
- **Battle-tested**: Most deployed database
- **Query power**: Complex filters, joins, etc.

### Why Both?
- JSONL is the **canonical format** (like source code)
- SQLite is the **index** (like compiled binary)
- Can rebuild SQLite from JSONL anytime

### Language: Why Rust?
- **Performance**: Sub-10ms queries
- **Reliability**: Type safety for spec compliance
- **Portability**: Single binary, no runtime
- **Ecosystem**: Great for CLI tools

## Performance Targets

- **Query latency**: < 10ms for local queries
- **Index build**: < 1s for 10k entries
- **Memory**: < 50MB for typical workload
- **Disk**: < 10MB per docpack (compressed)

## Open Questions

1. **Versioning**: How do we handle multiple versions of the same tool?
2. **Localization**: Should docpacks support translations?
3. **Media**: How to handle images, diagrams in docs?
4. **Private packs**: Authentication/authorization model?
5. **Compression**: Built into format or transport-level?

## Next Actions (Immediate)

1. ✅ Create specification document
2. ✅ Set up Rust project structure
3. ✅ Define core types
4. [ ] Implement JSONL reader
5. [ ] Implement basic query
6. [ ] Create first real docpack manually
7. [ ] Test query on real data

---

**Ship fast. Iterate faster. Make the standard stick.**
