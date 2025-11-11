# Contributing to LocalDoc

First off, **thank you** for considering contributing to LocalDoc! This project exists because developers like you believe documentation should be universally accessible.

## 🎯 Ways to Contribute

### 1. Create Docpacks
The easiest way to contribute! Package documentation for tools you use:

```bash
# Pack documentation
localdoc pack --source ./docs --name mytool --version 1.0.0 --output mytool.docpack

# Test it locally
localdoc query "something from mytool"

# Publish to Doctown (when registry launches)
localdoc publish mytool.docpack
```

**Popular tools that need docpacks:**
- Python frameworks (Django, Flask, FastAPI)
- JavaScript libraries (Vue, Svelte, Alpine.js)
- Rust crates (Tokio, Serde, Axum)
- CLI tools (kubectl, terraform, docker)
- Game engines (Bevy, Unity docs)

### 2. Improve the CLI
Found a bug? Have an idea for a feature?

```bash
# Fork and clone the repository
git clone https://github.com/xandwr/localdoc.git
cd localdoc

# Create a branch
git checkout -b feature/your-feature-name

# Make your changes
cargo build
cargo test

# Submit a pull request
git push origin feature/your-feature-name
```

### 3. Write Documentation
- Improve README or guides
- Write tutorials for creating docpacks
- Add code examples
- Fix typos or clarify confusing sections

### 4. Report Issues
Found a bug? [Open an issue](https://github.com/xandwr/localdoc/issues) with:
- Clear description of the problem
- Steps to reproduce
- Expected vs actual behavior
- Your environment (OS, Rust version)

### 5. Suggest Features
Have an idea? [Start a discussion](https://github.com/xandwr/localdoc/discussions) or open an issue tagged `enhancement`.

---

## 🛠️ Development Setup

### Prerequisites
- Rust 1.70+ (`rustup update`)
- Git
- A code editor (VS Code recommended)

### Quick Start
```bash
# Clone the repo
git clone https://github.com/xandwr/localdoc.git
cd localdoc

# Build the project
cargo build

# Run tests
cargo test

# Run the CLI locally
cargo run -- query "test"

# Check for issues
cargo clippy
cargo fmt --check
```

### Project Structure
```
localdoc/
├── src/
│   ├── main.rs           # CLI entry point
│   ├── docpack.rs        # Docpack data structures
│   ├── query.rs          # Search and query logic
│   ├── packer.rs         # Docpack creation
│   ├── godot_parser.rs   # Example: Godot XML parser
│   ├── lister.rs         # List installed docpacks
│   └── telemetry.rs      # Opt-in usage analytics
├── docpacks/             # Example docpacks
├── DOCPACK_SPEC.md       # Format specification
└── Cargo.toml            # Rust dependencies
```

## 🐛 Reporting Bugs

**Good bug reports include:**

1. **Clear title**: "Query crashes when searching for empty string"
2. **Environment**:
   ```
   - OS: Windows 11 / macOS 14 / Ubuntu 22.04
   - Rust version: 1.70.0
   - LocalDoc version: 0.1.0
   ```
3. **Steps to reproduce**:
   ```bash
   localdoc query ""
   ```
4. **Expected behavior**: "Should show helpful message"
5. **Actual behavior**: "Crashes with panic"
6. **Error output**: (paste full error)

---

## 📜 License

By contributing, you agree that your contributions will be licensed under:
- **Implementation**: Apache 2.0
- **Specification**: CC0 (Public Domain)

This means:
- Your code contributions are Apache 2.0
- Documentation/spec contributions are public domain
- You retain copyright, but grant usage rights

---

## 📬 Questions?
- **GitHub Discussions**: Ask anything
- **Email**: Reach out to hello@doctown.dev