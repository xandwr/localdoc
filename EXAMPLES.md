# Localdoc Usage Examples

This document provides practical examples of using localdoc to explore documentation.

## Basic Inspection

Check what's in a docpack:

```bash
$ localdoc inspect ../builder/doctown-builder-docs.docpack

Docpack Metadata
==================================================

Format Version: 1

Project Information:
  Name: doctown-builder-docs
  Version: 0.1.0

Generated At: 2025-11-17T07:38:55.846854877+00:00

Language Summary:
  rust_files: 60

Statistics:
  Symbols Extracted: 60
  Docs Generated: 60
```

## Finding What You Need

### List all files to understand the codebase structure

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack files

Source Files
==================================================

xandwr-doctown-builder-7cb235c/src/bin/ingest_zip.rs (1 symbols)
xandwr-doctown-builder-7cb235c/src/config.rs (3 symbols)
xandwr-doctown-builder-7cb235c/src/error.rs (13 symbols)
xandwr-doctown-builder-7cb235c/src/main.rs (2 symbols)
...
```

### Explore a specific file

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack file config.rs

Symbols in 'config.rs'
==================================================

[struct] xandwr-doctown-builder-7cb235c/src/config.rs:5:Config (line 5)
  pub struct Config {

[impl] xandwr-doctown-builder-7cb235c/src/config.rs:12:Config (line 12)
  impl Config {

[function] xandwr-doctown-builder-7cb235c/src/config.rs:13:from_env (line 13)
  pub fn from_env() -> Result<Self> {
```

## Deep Diving into Documentation

### Get complete documentation for a symbol

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack symbol from_env

Symbol Information
==================================================

ID: xandwr-doctown-builder-7cb235c/src/config.rs:13:from_env
Kind: function
File: xandwr-doctown-builder-7cb235c/src/config.rs:13
Signature: pub fn from_env() -> Result<Self> {

Documentation
--------------------------------------------------

Summary: Reads OpenAI configuration from environment variables...

Description:
from_env is a constructor-like function for the configuration object...

Returns: Result<Self> indicating either a successfully constructed...

Example:
use your_crate::config::Config;

// Initialize configuration from environment variables
let config = Config::from_env()?;

Notes:
  - Requires OPENAI_API_KEY to be set in the environment
  - OPENAI_MODEL is optional; if not provided, the default 'gpt-4o-mini' is used
  ...
```

## Searching and Discovery

### Search for error handling code

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack search "error"

Search Results for 'error'
==================================================

[enum] xandwr-doctown-builder-7cb235c/src/error.rs:4:Error
  Location: xandwr-doctown-builder-7cb235c/src/error.rs:4
  Summary: Error represents all possible error conditions that can occur...

[impl] xandwr-doctown-builder-7cb235c/src/error.rs:27:Error
  Location: xandwr-doctown-builder-7cb235c/src/error.rs:27
  Summary: Implements Display for the Error enum, enabling human-readable...
  
...
```

### Search for configuration-related code

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack search Config
```

### Find parsing functions

```bash
$ localdoc query ../builder/doctown-builder-docs.docpack search parse
```

## Pro Tips

### Pipe to grep for further filtering

```bash
# Find all function symbols
localdoc query docs.docpack symbols | grep "\[function\]"

# Find symbols in a specific directory
localdoc query docs.docpack symbols | grep "src/pipeline/"
```

### Pipe to less for browsing large outputs

```bash
localdoc query docs.docpack symbols | less
```

### Save documentation to a file

```bash
localdoc query docs.docpack symbol "my_function" > my_function_docs.txt
```

### Use in scripts

```bash
#!/bin/bash
# Check if a symbol exists
if localdoc query docs.docpack symbol "init" > /dev/null 2>&1; then
    echo "Found init function"
else
    echo "init function not found"
fi
```

## Integration with Your Workflow

### Pre-commit hook to check documentation

```bash
# .git/hooks/pre-commit
#!/bin/bash
./builder/target/release/doctown-builder .
./localdoc/target/release/localdoc inspect output.docpack
```

### CI/CD Pipeline

```yaml
# .github/workflows/docs.yml
- name: Generate docs
  run: cargo run --manifest-path builder/Cargo.toml -- .
  
- name: Validate docpack
  run: |
    cd localdoc
    cargo run -- inspect ../output.docpack
```

### VS Code Integration

Add to your `tasks.json`:

```json
{
    "label": "Query Symbol",
    "type": "shell",
    "command": "./localdoc/target/release/localdoc query docs.docpack symbol ${input:symbolName}",
    "inputs": [
        {
            "id": "symbolName",
            "type": "promptString",
            "description": "Symbol name to query"
        }
    ]
}
```
