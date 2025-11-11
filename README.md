# localdoc

> **npm for documentation** — The universal standard for packaging, sharing, and querying developer documentation

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Spec License](https://img.shields.io/badge/Spec-CC0-green.svg)](DOCPACK_SPEC.md)

### 🎯 The Vision
**The CLI will always be free and open source.** Doctown Registry enables:
- Publish and share docpacks with the community
- Install any documentation with one command
- Discover docpacks for any tool, language, or framework
- Verified badges for official maintainers
- Private docpacks for teams and enterprises

---

## 🚀 What is LocalDoc?

`localdoc` is the command-line tool that makes **any documentation** instantly queryable from your terminal—no internet required once installed.

**Doctown Registry** is the npm-like platform where developers publish, discover, and install `.docpack` files.

Together, they solve the documentation fragmentation problem across the entire developer ecosystem.

---

## 💥 The Problem

Every tool, language, and framework has its own documentation format:
- Rust has rustdoc HTML
- Python has Sphinx
- JavaScript has JSDoc
- CLI tools have man pages and --help
- APIs have OpenAPI specs
- Game engines have custom wikis

**As developers, we waste time:**
- Context-switching between different doc sites
- Remembering different search interfaces
- Dealing with slow web searches
- Working offline without access to docs

---

## ✨ The Solution: `.docpack` Format + Doctown Registry

### The `.docpack` Format
A universal, open standard for packaging documentation:

✅ **Minimal** - Simple JSON-based format, no bloat  
✅ **Consistent** - Same structure across all tools  
✅ **Queryable** - Optimized for fast local search  
✅ **Serializable** - Works as SQLite, JSONL, or binary  
✅ **Composable** - Docpacks reference each other  
✅ **Open Spec** - Anyone can implement or create docpacks

### Doctown Registry
A community-driven platform for sharing docpacks:

✅ **Publish** - Share your project's documentation  
✅ **Install** - One command to get any documentation  
✅ **Verify** - Official badges for maintainers  
✅ **Discover** - Browse thousands of docpacks  
✅ **Private** - Team/enterprise private docpacks  

---

## 🎬 Quick Start

```bash
# Install the CLI (coming soon: cargo install localdoc)
cargo install localdoc

# Install documentation for any tool
localdoc install rust
localdoc install python
localdoc install godot

# Query instantly from your terminal
localdoc query "how to iterate over a vector"
localdoc query "django authentication"
localdoc query "godot Vector3"

# Create your own docpack
localdoc pack --source ./docs --name myproject --version 1.0.0

# Publish to Doctown Registry
localdoc publish myproject.docpack

# Everything is local, fast, and works offline
```

---

## 🌟 Key Features

### For Developers
- 🔍 **Lightning-fast local search** - Query 100k+ docs in milliseconds
- 🌐 **Works offline** - All docs stored locally
- 🔄 **Universal interface** - Same commands for any tool
- 🎨 **Multiple outputs** - Terminal, JSON, or IDE integration
- ⚡ **Zero config** - Works out of the box

### For Tool Maintainers
- 📦 **Easy publishing** - One command to share your docs
- ✅ **Verified badges** - Prove official ownership via GitHub
- 📊 **Analytics** - See who's using your documentation
- 🔗 **Discoverability** - Listed on Doctown Registry
- 🆓 **Free forever** - Open source, community-driven

### For Teams & Enterprises
- 🔒 **Private docpacks** - Internal tools documentation
- 👥 **Team sharing** - Invite members, control access
- 🏢 **Self-hosted option** - Run your own registry
- 📈 **Usage analytics** - Track documentation adoption
- 🤝 **Enterprise support** - SLA and priority assistance

---

## 🏗️ Current Status

**✅ Working Now:**
- LocalDoc CLI with query, pack, list, show commands
- Fuzzy search with intelligent scoring
- 15,978+ Godot 4.5 documentation entries indexed
- `.docpack` specification v0.1.0
- JSONL and SQLite storage backends

**🔨 In Development (Next 8 Weeks):**
- Doctown Registry (API + Web UI)
- `localdoc install/publish/search` commands
- GitHub OAuth authentication
- VSCode extension for inline documentation
- 10+ essential docpacks (Python, Node.js, React, Rust, Go, etc.)
- Pro/Team tier subscriptions

---

## 🎯 Roadmap

### Phase 1: Foundation (Weeks 1-2) ✅
- [x] Core CLI implementation
- [x] `.docpack` specification
- [x] Godot docpack generation
- [ ] Fix compiler warnings
- [ ] Add comprehensive tests
- [ ] Improved error handling

### Phase 2: Registry Launch (Weeks 3-6)
- [ ] Doctown Registry API (Rust/Axum)
- [ ] GitHub OAuth integration
- [ ] Web UI for browsing/publishing
- [ ] CLI commands: install, publish, search
- [ ] Documentation and guides

### Phase 3: Ecosystem Growth (Weeks 7-8)
- [ ] VSCode extension
- [ ] 10+ popular docpacks
- [ ] Community tools and integrations
- [ ] Pro/Team tier launch

### Phase 4: Scale (Months 3-6)
- [ ] Private docpack support
- [ ] Analytics dashboard
- [ ] API for third-party integrations
- [ ] Self-hosted registry option
- [ ] Enterprise features

---

## 💰 Sustainability Model

**LocalDoc CLI**: Free and open source, always.

**Doctown Registry Free Tier:**
- ✅ Install unlimited docpacks
- ✅ Publish up to 3 docpacks
- ✅ Browse and search all public docpacks

**Doctown Pro ($5/month):**
- ✅ Publish unlimited docpacks
- ✅ Verified badge (GitHub-linked)
- ✅ Analytics for your docpacks
- ✅ Priority indexing

**Doctown Team ($50/month):**
- ✅ Everything in Pro
- ✅ Private docpacks
- ✅ Team sharing and access control
- ✅ SSO integration

**Doctown Enterprise (Custom):**
- ✅ Self-hosted registry
- ✅ Priority support & SLA
- ✅ Custom integrations
- ✅ Dedicated infrastructure

All revenue goes toward:
- Infrastructure and hosting costs
- Maintaining the open-source CLI
- Building new features
- Supporting the community

**No ads. No tracking. No dark patterns.**

---

## 🤝 Contributing

We're building this in public. Contributions welcome!

**Ways to contribute:**
- Create docpacks for your favorite tools
- Improve the CLI (bug fixes, features, tests)
- Write documentation and guides
- Report issues and suggest features
- Help others in discussions

See [CONTRIBUTING.md](./CONTRIBUTING.md) for details.

---

## 📚 Documentation

- [`.docpack` Specification](./DOCPACK_SPEC.md) - Technical format details
- [Creating Docpacks](./docs/creating-docpacks.md) - Guide for packaging docs
- [Funding & Roadmap](./FUNDING_ROADMAP.md) - Sustainability model
- [API Reference](./docs/api.md) - Doctown Registry API

---

## 🌍 Why This Matters

Documentation is infrastructure. But it's fragmented, inconsistent, and often inaccessible when you need it most.

**Doctown makes documentation a solved problem:**
- Developers get instant access to any documentation
- Tool maintainers get a standard way to distribute docs
- The community builds a shared knowledge commons

This is how documentation should work. Universal, fast, local, and open.

---

## 📜 License

- **Specification**: CC0 (Public Domain) - anyone can implement
- **Implementation**: Apache 2.0 - free to use, modify, and distribute

See [LICENSE](./LICENSE) for full details.

---

## 🚀 Get Started

```bash
# Clone and build
git clone https://github.com/xandwr/localdoc.git
cd localdoc
cargo build --release

# Try it now with the included Godot docpack
./target/release/localdoc query "Vector3"

# Join us in building the future of developer documentation
```

**Questions? Ideas?** Open an issue or discussion on GitHub.

**Want to help?** Check out [CONTRIBUTING.md](./CONTRIBUTING.md) and pick up a "good first issue".