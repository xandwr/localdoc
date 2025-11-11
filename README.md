# localdoc

> The universal standard for local tool documentation

### Note:
**The CLI will always be free and open source.** Doctown is optional for those who want:
- Remote queries (no local installation)
- Community docpack repository
- Instant access to curated documentation

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

## Status

**Early Development** - This is the foundational work. We're building:
1. The `.docpack` specification (v0.1.0)
2. The reference implementation (`localdoc` CLI)
3. Tooling for generating docpacks
4. A registry for discovering and downloading docpacks

## Contributing

The `.docpack` spec is open. The implementation is open source. 

If you're building dev tools, you should care about this. If you're tired of fragmented documentation, you should care about this.

See [DOCPACK_SPEC.md](./DOCPACK_SPEC.md) for the full specification.