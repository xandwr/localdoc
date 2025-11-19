use crate::docpack::Docpack;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

// JSON-RPC 2.0 types
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

#[derive(Debug, Serialize)]
struct Tool {
    name: String,
    description: String,
    #[serde(rename = "inputSchema")]
    input_schema: Value,
}

pub struct McpServer {
    packages_dir: PathBuf,
}

impl McpServer {
    pub fn new(packages_dir: PathBuf) -> Self {
        McpServer { packages_dir }
    }

    pub fn run(&self) -> Result<()> {
        let stdin = std::io::stdin();
        let mut stdout = std::io::stdout();
        let reader = BufReader::new(stdin.lock());

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let request: JsonRpcRequest = match serde_json::from_str(&line) {
                Ok(req) => req,
                Err(e) => {
                    let error_response = JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError {
                            code: -32700,
                            message: format!("Parse error: {}", e),
                            data: None,
                        }),
                    };
                    writeln!(stdout, "{}", serde_json::to_string(&error_response)?)?;
                    stdout.flush()?;
                    continue;
                }
            };

            let response = self.handle_request(request);
            writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
            stdout.flush()?;
        }

        Ok(())
    }

    fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let id = request.id.unwrap_or(Value::Null);

        let result = match request.method.as_str() {
            "initialize" => self.handle_initialize(),
            "initialized" => Ok(json!({})),
            "tools/list" => self.handle_tools_list(),
            "tools/call" => self.handle_tools_call(&request.params),
            "ping" => Ok(json!({})),
            _ => Err(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
                data: None,
            }),
        };

        match result {
            Ok(result) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            },
            Err(error) => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: None,
                error: Some(error),
            },
        }
    }

    fn handle_initialize(&self) -> Result<Value, JsonRpcError> {
        Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "localdoc",
                "version": env!("CARGO_PKG_VERSION")
            }
        }))
    }

    fn handle_tools_list(&self) -> Result<Value, JsonRpcError> {
        let tools = vec![
            Tool {
                name: "list_packages".to_string(),
                description: "List all installed docpacks".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {},
                    "required": []
                }),
            },
            Tool {
                name: "list_symbols".to_string(),
                description: "List all symbols in a docpack".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "package": {
                            "type": "string",
                            "description": "Package name in format username:reponame"
                        }
                    },
                    "required": ["package"]
                }),
            },
            Tool {
                name: "get_symbol".to_string(),
                description: "Get full documentation for a specific symbol".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "package": {
                            "type": "string",
                            "description": "Package name in format username:reponame"
                        },
                        "symbol": {
                            "type": "string",
                            "description": "Symbol name or ID to look up"
                        }
                    },
                    "required": ["package", "symbol"]
                }),
            },
            Tool {
                name: "search".to_string(),
                description: "Search for symbols across docpacks by keyword".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Search query"
                        },
                        "package": {
                            "type": "string",
                            "description": "Optional: limit search to specific package"
                        }
                    },
                    "required": ["query"]
                }),
            },
        ];

        Ok(json!({ "tools": tools }))
    }

    fn handle_tools_call(&self, params: &Value) -> Result<Value, JsonRpcError> {
        let name = params["name"].as_str().ok_or_else(|| JsonRpcError {
            code: -32602,
            message: "Missing tool name".to_string(),
            data: None,
        })?;

        let arguments = &params["arguments"];

        let result = match name {
            "list_packages" => self.tool_list_packages(),
            "list_symbols" => self.tool_list_symbols(arguments),
            "get_symbol" => self.tool_get_symbol(arguments),
            "search" => self.tool_search(arguments),
            _ => Err(format!("Unknown tool: {}", name)),
        };

        match result {
            Ok(text) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": text
                }]
            })),
            Err(e) => Ok(json!({
                "content": [{
                    "type": "text",
                    "text": e
                }],
                "isError": true
            })),
        }
    }

    fn tool_list_packages(&self) -> Result<String, String> {
        if !self.packages_dir.exists() {
            return Ok("No docpacks installed yet.".to_string());
        }

        let entries: Vec<_> = std::fs::read_dir(&self.packages_dir)
            .map_err(|e| format!("Failed to read packages directory: {}", e))?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|ext| ext == "docpack")
                    .unwrap_or(false)
            })
            .collect();

        if entries.is_empty() {
            return Ok("No docpacks installed yet.".to_string());
        }

        let mut output = String::new();
        output.push_str("Installed docpacks:\n\n");

        for entry in &entries {
            let path = entry.path();
            let filename = path.file_stem().unwrap_or_default().to_string_lossy();
            let name = filename.replacen('_', ":", 1);

            match Docpack::open(&path.to_string_lossy()) {
                Ok(docpack) => {
                    output.push_str(&format!(
                        "- {} (v{}, {} symbols)\n",
                        name,
                        docpack.manifest.project.version,
                        docpack.manifest.stats.symbols_extracted
                    ));
                }
                Err(_) => {
                    output.push_str(&format!("- {} (unable to read metadata)\n", name));
                }
            }
        }

        Ok(output)
    }

    fn tool_list_symbols(&self, args: &Value) -> Result<String, String> {
        let package = args["package"]
            .as_str()
            .ok_or("Missing 'package' argument")?;

        let path = self.resolve_package_path(package)?;
        let docpack = Docpack::open(&path).map_err(|e| format!("Failed to open docpack: {}", e))?;

        let mut output = String::new();
        output.push_str(&format!("Symbols in {}:\n\n", package));

        for symbol in &docpack.symbols {
            output.push_str(&format!(
                "[{}] {} ({}:{})\n",
                symbol.kind, symbol.id, symbol.file, symbol.line
            ));
        }

        output.push_str(&format!("\nTotal: {} symbols", docpack.symbols.len()));
        Ok(output)
    }

    fn tool_get_symbol(&self, args: &Value) -> Result<String, String> {
        let package = args["package"]
            .as_str()
            .ok_or("Missing 'package' argument")?;
        let symbol_name = args["symbol"]
            .as_str()
            .ok_or("Missing 'symbol' argument")?;

        let path = self.resolve_package_path(package)?;
        let mut docpack =
            Docpack::open(&path).map_err(|e| format!("Failed to open docpack: {}", e))?;

        let matches: Vec<_> = docpack
            .find_symbols_by_name(symbol_name)
            .into_iter()
            .cloned()
            .collect();

        if matches.is_empty() {
            return Err(format!("No symbol found matching '{}'", symbol_name));
        }

        let mut output = String::new();

        for symbol in matches {
            let doc = docpack
                .get_documentation(&symbol.doc_id)
                .map_err(|e| format!("Failed to get documentation: {}", e))?;

            output.push_str(&format!("# {}\n\n", symbol.id));
            output.push_str(&format!("**Kind:** {}\n", symbol.kind));
            output.push_str(&format!("**Location:** {}:{}\n", symbol.file, symbol.line));
            output.push_str(&format!("**Signature:** `{}`\n\n", symbol.signature));

            output.push_str(&format!("## Summary\n{}\n\n", doc.summary));
            output.push_str(&format!("## Description\n{}\n\n", doc.description));

            if !doc.parameters.is_empty() {
                output.push_str("## Parameters\n");
                for param in &doc.parameters {
                    output.push_str(&format!(
                        "- **{}** ({}): {}\n",
                        param.name, param.param_type, param.description
                    ));
                }
                output.push('\n');
            }

            if !doc.returns.is_empty() {
                output.push_str(&format!("## Returns\n{}\n\n", doc.returns));
            }

            if !doc.example.is_empty() {
                output.push_str(&format!("## Example\n```\n{}\n```\n\n", doc.example));
            }

            if !doc.notes.is_empty() {
                output.push_str("## Notes\n");
                for note in &doc.notes {
                    output.push_str(&format!("- {}\n", note));
                }
                output.push('\n');
            }

            output.push_str("---\n\n");
        }

        Ok(output)
    }

    fn tool_search(&self, args: &Value) -> Result<String, String> {
        let query = args["query"].as_str().ok_or("Missing 'query' argument")?;
        let package_filter = args["package"].as_str();

        let mut all_results: Vec<(String, String, String, String)> = Vec::new();

        if let Some(package) = package_filter {
            // Search specific package
            let path = self.resolve_package_path(package)?;
            let mut docpack =
                Docpack::open(&path).map_err(|e| format!("Failed to open docpack: {}", e))?;

            let results = docpack
                .search_symbols(query)
                .map_err(|e| format!("Search failed: {}", e))?;

            for (symbol, doc) in results {
                all_results.push((
                    package.to_string(),
                    symbol.id,
                    symbol.kind,
                    doc.summary,
                ));
            }
        } else {
            // Search all packages
            if self.packages_dir.exists() {
                let entries: Vec<_> = std::fs::read_dir(&self.packages_dir)
                    .map_err(|e| format!("Failed to read packages directory: {}", e))?
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "docpack")
                            .unwrap_or(false)
                    })
                    .collect();

                for entry in entries {
                    let path = entry.path();
                    let filename = path.file_stem().unwrap_or_default().to_string_lossy();
                    let package_name = filename.replacen('_', ":", 1);

                    if let Ok(mut docpack) = Docpack::open(&path.to_string_lossy()) {
                        if let Ok(results) = docpack.search_symbols(query) {
                            for (symbol, doc) in results {
                                all_results.push((
                                    package_name.clone(),
                                    symbol.id,
                                    symbol.kind,
                                    doc.summary,
                                ));
                            }
                        }
                    }
                }
            }
        }

        if all_results.is_empty() {
            return Ok(format!("No results found for '{}'", query));
        }

        let mut output = String::new();
        output.push_str(&format!("Search results for '{}':\n\n", query));

        for (package, id, kind, summary) in &all_results {
            output.push_str(&format!("[{}] {}:{}\n", kind, package, id));
            output.push_str(&format!("  {}\n\n", summary));
        }

        output.push_str(&format!("Found {} result(s)", all_results.len()));
        Ok(output)
    }

    fn resolve_package_path(&self, package: &str) -> Result<String, String> {
        let filename = format!("{}.docpack", package.replace(':', "_"));
        let path = self.packages_dir.join(&filename);

        if path.exists() {
            Ok(path.to_string_lossy().to_string())
        } else {
            Err(format!(
                "Docpack '{}' not found. Run 'localdoc list' to see installed docpacks.",
                package
            ))
        }
    }
}
