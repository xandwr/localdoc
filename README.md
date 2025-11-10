# localdoc

> The universal standard for local tool documentation

## What is this?

`localdoc` is not just another documentation tool. It's the **foundation** for how developers will access, query, and use documentation locally across all ecosystems.

## The Problem

Every tool, language, and framework has its own documentation format:
- Rust has rustdoc HTML
- Python has Sphinx
- JavaScript has JSDoc
- CLI tools have man pages and --help
- APIs have OpenAPI specs

As developers, we constantly context-switch between tools. We need to remember different doc systems, search in different places, and deal with inconsistent formats.

## The Solution: `.docpack`

The `.docpack` format is a universal standard for packaging documentation:

✅ **Minimal** - Simple JSON-based format, no bloat  
✅ **Consistent** - Same structure across all tools  
✅ **Queryable** - Optimized for fast local search  
✅ **Serializable** - Works as SQLite, JSONL, or binary  
✅ **Composable** - Docpacks reference each other  

## Vision

Imagine a world where:
```bash
# Install any tool's documentation
localdoc install rust
localdoc install django
localdoc install git

# Query across all of them with the same interface
localdoc query "how to iterate over a vector"
localdoc query "django authentication"
localdoc query "git rebase"

# Everything is local, fast, and consistent
```

## The Rosetta Stone

`.docpack` is the Rosetta Stone between ecosystems:
- **Publishers**: Tool maintainers ship `.docpack` files alongside their tools
- **Consumers**: Developers query them with `localdoc`
- **Integrators**: IDEs, editors, and AI tools consume the standard format

## Status

🚧 **Early Development** - This is the foundational work. We're building:
1. ✅ The `.docpack` specification (v0.1.0)
2. 🚧 The reference implementation (`localdoc` CLI)
3. 🔜 Tooling for generating docpacks
4. 🔜 A registry for discovering and downloading docpacks

## Quick Start

```bash
# Install
cargo install localdoc

# It auto-initializes on first run
localdoc

# Documentation lives in ~/localdoc
```

## Architecture

```
~/localdoc/
├── packs/              # Installed docpacks
│   ├── rust-std.docpack
│   ├── python-stdlib.docpack
│   └── git.docpack
├── index.db           # Unified search index
└── config.toml        # User preferences
```

## Core Principles

1. **Local First** - No internet required once installed
2. **Fast** - Optimized for sub-10ms queries
3. **Offline AI** - Perfect for local LLMs and IDE integrations
4. **Open Standard** - The spec is public domain (CC0)
5. **Ecosystem Agnostic** - Works for any tool, language, or framework

## The Play

This isn't just a CLI tool. This is **infrastructure**.

When `.docpack` becomes the standard:
- Tool maintainers will ship docpacks
- IDE vendors will consume docpacks
- AI coding assistants will train on docpacks
- Enterprise teams will create internal docpacks

We're not competing with existing doc tools. We're creating the **interoperability layer**.

## Contributing

The `.docpack` spec is open. The implementation is open source. 

If you're building dev tools, you should care about this. If you're tired of fragmented documentation, you should care about this.

See [DOCPACK_SPEC.md](./DOCPACK_SPEC.md) for the full specification.

## License

- Specification: CC0 (Public Domain)
- Implementation: MIT OR Apache-2.0

---

**This is the standard. Let's build it together.**
