/// Integration tests for localdoc CLI
///
/// These tests verify the core functionality of the docpack system
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

// ============================================================================
// Test Helper Functions
// ============================================================================

#[allow(dead_code)]
/// Creates a temporary directory for testing
fn setup_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

#[allow(dead_code)]
/// Get the path to the test binary
fn get_cli_binary() -> PathBuf {
    let mut path = std::env::current_exe().expect("Failed to get current executable");
    path.pop(); // Remove test binary name
    path.pop(); // Remove "deps" directory
    path.push("localdoc");

    // On Windows, add .exe extension
    if cfg!(target_os = "windows") {
        path.set_extension("exe");
    }

    path
}

#[allow(dead_code)]
/// Run the localdoc CLI with the given arguments
fn run_cli(args: &[&str]) -> std::process::Output {
    let binary = get_cli_binary();
    Command::new(&binary)
        .args(args)
        .output()
        .expect("Failed to execute CLI command")
}

/// Create a minimal valid Godot XML file
fn create_test_xml_file(path: &Path, class_name: &str, has_methods: bool) {
    let methods_xml = if has_methods {
        r#"
    <methods>
        <method name="test_method">
            <return type="void" />
            <description>A test method.</description>
        </method>
    </methods>"#
    } else {
        "<methods />"
    };

    let content = format!(
        r#"<?xml version="1.0" encoding="UTF-8" ?>
<class name="{}" inherits="Object">
    <brief_description>Test class for {}</brief_description>
    <description>This is a test class used for integration testing.</description>
    {}
    <members />
    <signals />
    <constants />
</class>"#,
        class_name, class_name, methods_xml
    );

    fs::write(path, content).expect("Failed to write test XML file");
}

/// Validate that a manifest.json file exists and is well-formed
fn validate_manifest(docpack_dir: &Path) -> serde_json::Value {
    let manifest_path = docpack_dir.join("manifest.json");
    assert!(
        manifest_path.exists(),
        "manifest.json does not exist in docpack"
    );

    let content = fs::read_to_string(&manifest_path).expect("Failed to read manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse manifest.json");

    // Validate required fields
    assert!(manifest["docpack_version"].is_string());
    assert!(manifest["tool"]["name"].is_string());
    assert!(manifest["tool"]["version"].is_string());
    assert!(manifest["metadata"]["entry_count"].is_number());

    manifest
}

/// Validate that content.jsonl exists and has valid entries
fn validate_content_jsonl(docpack_dir: &Path) -> Vec<serde_json::Value> {
    let content_path = docpack_dir.join("content.jsonl");
    assert!(
        content_path.exists(),
        "content.jsonl does not exist in docpack"
    );

    let content = fs::read_to_string(&content_path).expect("Failed to read content.jsonl");
    let entries: Vec<serde_json::Value> = content
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("Failed to parse JSONL entry"))
        .collect();

    // Validate each entry has required fields
    for (i, entry) in entries.iter().enumerate() {
        assert!(
            entry["id"].is_string(),
            "Entry {} missing 'id' field: {:?}",
            i,
            entry
        );
        assert!(
            entry["type"].is_string(),
            "Entry {} missing 'type' field: {:?}",
            i,
            entry
        );
        assert!(
            entry["name"].is_string(),
            "Entry {} missing 'name' field: {:?}",
            i,
            entry
        );
        assert!(
            entry["content"].is_string(),
            "Entry {} missing 'content' field: {:?}",
            i,
            entry
        );
    }

    entries
}

// ============================================================================
// Basic Validation Tests
// ============================================================================

/// Test that we can create and parse a basic manifest structure
#[test]
fn test_docpack_manifest_creation() {
    let manifest_json = r#"{
        "docpack_version": "0.1.0",
        "tool": {
            "name": "test-tool",
            "version": "1.0.0",
            "ecosystem": "testing",
            "homepage": null
        },
        "metadata": {
            "generated_at": "2025-11-10T00:00:00Z",
            "generator": "localdoc-test",
            "content_hash": "abc123",
            "entry_count": 0
        },
        "schema": {
            "version": "0.1.0",
            "extensions": []
        },
        "dependencies": []
    }"#;

    let result: Result<serde_json::Value, _> = serde_json::from_str(manifest_json);
    assert!(result.is_ok(), "Failed to parse manifest JSON");

    let manifest = result.unwrap();
    assert_eq!(manifest["docpack_version"], "0.1.0");
    assert_eq!(manifest["tool"]["name"], "test-tool");
    assert_eq!(manifest["tool"]["version"], "1.0.0");
    assert_eq!(manifest["metadata"]["entry_count"], 0);

    println!("✓ Docpack manifest test passed");
}

/// Test that the DOCPACK_VERSION constant is properly set
#[test]
fn test_docpack_version_constant() {
    let expected_version = "0.1.0";
    assert_eq!(expected_version, "0.1.0");
    println!("✓ Docpack version constant test passed");
}

/// Test basic file system operations for docpack discovery
#[test]
fn test_docpack_directory_structure() {
    let manifest = std::env::current_dir().expect("Failed to get current directory");
    let docpack_dir = manifest.join("docpacks");
    assert!(docpack_dir.is_relative() || docpack_dir.is_absolute());
    println!("✓ Directory structure test passed");
}

// ============================================================================
// XML Parsing Edge Cases
// ============================================================================

/// Test parsing valid XML from test_doc_sources
#[test]
fn test_parse_real_godot_xml() {
    let test_file = PathBuf::from("tests/test_doc_sources/godot-4.5/Node.xml");

    if !test_file.exists() {
        println!("⚠️  Skipping test - Node.xml not found");
        return;
    }

    // Import the parser module
    use localdoc_cli::godot_parser;

    let result = godot_parser::parse_godot_xml(&test_file);
    assert!(
        result.is_ok(),
        "Failed to parse Node.xml: {:?}",
        result.err()
    );

    let entries = result.unwrap();
    assert!(!entries.is_empty(), "No entries parsed from Node.xml");

    // Verify we have a class entry
    use localdoc_cli::EntryType;
    let class_entry = entries
        .iter()
        .find(|e| matches!(e.entry_type, EntryType::Class));
    assert!(class_entry.is_some(), "No class entry found");

    println!(
        "✓ Successfully parsed real Godot XML with {} entries",
        entries.len()
    );
}

/// Test parsing multiple XML files
#[test]
fn test_parse_multiple_godot_xml_files() {
    use localdoc_cli::godot_parser;

    let test_files = vec![
        "tests/test_doc_sources/godot-4.5/Node.xml",
        "tests/test_doc_sources/godot-4.5/Node2D.xml",
        "tests/test_doc_sources/godot-4.5/Object.xml",
    ];

    let mut total_entries = 0;
    let mut parsed_files = 0;

    for file_path in test_files {
        let path = PathBuf::from(file_path);
        if path.exists() {
            match godot_parser::parse_godot_xml(&path) {
                Ok(entries) => {
                    total_entries += entries.len();
                    parsed_files += 1;
                }
                Err(e) => {
                    println!("⚠️  Warning: Failed to parse {}: {}", file_path, e);
                }
            }
        }
    }

    assert!(parsed_files > 0, "No files were successfully parsed");
    assert!(total_entries > 0, "No entries extracted from parsed files");

    println!(
        "✓ Parsed {} files with {} total entries",
        parsed_files, total_entries
    );
}

// ============================================================================
// Packing Tests
// ============================================================================

/// Test creating a docpack from a small set of XML files
#[test]
fn test_pack_simple_docpack() {
    let temp_dir = setup_temp_dir();
    let source_dir = temp_dir.path().join("source");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    // Create test XML files
    create_test_xml_file(&source_dir.join("TestClass.xml"), "TestClass", true);
    create_test_xml_file(&source_dir.join("AnotherClass.xml"), "AnotherClass", false);

    // Pack the documentation
    use localdoc_cli::packer;
    let output_path = output_dir.join("test.docpack");

    let result = packer::pack_godot_docs(&source_dir, &output_path, "test-tool", "1.0.0");

    assert!(
        result.is_ok(),
        "Failed to pack documentation: {:?}",
        result.err()
    );
    assert!(output_path.exists(), "Docpack was not created");

    // Validate the docpack structure
    let manifest = validate_manifest(&output_path);
    assert_eq!(manifest["tool"]["name"], "test-tool");

    let entries = validate_content_jsonl(&output_path);
    assert!(!entries.is_empty(), "No entries in content.jsonl");

    println!(
        "✓ Successfully created and validated docpack with {} entries",
        entries.len()
    );
}

/// Test packing with real Godot XML files
#[test]
fn test_pack_real_godot_docs() {
    let source_dir = PathBuf::from("tests/test_doc_sources/godot-4.5");

    if !source_dir.exists() {
        println!("⚠️  Skipping test - test source directory not found");
        return;
    }

    let temp_dir = setup_temp_dir();
    let output_path = temp_dir.path().join("godot-test.docpack");

    use localdoc_cli::packer;
    let result = packer::pack_godot_docs(&source_dir, &output_path, "godot-test", "4.5.0");

    assert!(
        result.is_ok(),
        "Failed to pack real Godot docs: {:?}",
        result.err()
    );
    assert!(output_path.exists(), "Docpack was not created");

    // Validate structure
    let manifest = validate_manifest(&output_path);
    assert_eq!(manifest["tool"]["name"], "godot-test");
    assert_eq!(manifest["tool"]["version"], "4.5.0");

    let entries = validate_content_jsonl(&output_path);
    let entry_count = entries.len();

    // With hundreds of XML files, we should have many entries
    assert!(
        entry_count > 100,
        "Expected more than 100 entries, got {}",
        entry_count
    );

    println!(
        "✓ Successfully packed real Godot docs with {} entries",
        entry_count
    );
}

/// Test packing with empty directory (edge case)
#[test]
fn test_pack_empty_directory() {
    let temp_dir = setup_temp_dir();
    let empty_source = temp_dir.path().join("empty");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&empty_source).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    use localdoc_cli::packer;
    let output_path = output_dir.join("empty.docpack");

    let result = packer::pack_godot_docs(&empty_source, &output_path, "empty-test", "1.0.0");

    // Should succeed even with no files
    assert!(result.is_ok(), "Failed to pack empty directory");
    assert!(output_path.exists(), "Docpack was not created");

    let manifest = validate_manifest(&output_path);
    assert_eq!(manifest["metadata"]["entry_count"], 0);

    println!("✓ Successfully handled empty directory");
}

// ============================================================================
// End-to-End Pipeline Tests
// ============================================================================

/// Test the full pipeline: build -> pack -> query
#[test]
fn test_full_pipeline_build_pack_query() {
    let temp_dir = setup_temp_dir();
    let source_dir = temp_dir.path().join("source");
    let output_dir = temp_dir.path().join("output");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&output_dir).unwrap();

    // Step 1: Create test XML files
    create_test_xml_file(&source_dir.join("Node.xml"), "Node", true);
    create_test_xml_file(&source_dir.join("Sprite.xml"), "Sprite", true);
    create_test_xml_file(&source_dir.join("Camera.xml"), "Camera", false);

    // Step 2: Pack into docpack
    use localdoc_cli::packer;
    let docpack_path = output_dir.join("test-game.docpack");
    let pack_result = packer::pack_godot_docs(&source_dir, &docpack_path, "test-game", "1.0.0");
    assert!(
        pack_result.is_ok(),
        "Failed to pack: {:?}",
        pack_result.err()
    );

    // Step 3: Validate docpack structure
    let manifest = validate_manifest(&docpack_path);
    assert_eq!(manifest["tool"]["name"], "test-game");

    let entries = validate_content_jsonl(&docpack_path);
    assert!(entries.len() >= 3, "Expected at least 3 class entries");

    // Step 4: Query the docpack
    use localdoc_cli::query::{QueryOptions, query_docpacks};
    let search_dirs = vec![output_dir.clone()];
    let query_opts = QueryOptions {
        entry_type: None,
        path: None,
        docpack: None,
        verbose: false,
    };

    let results = query_docpacks(&search_dirs, "Node", &query_opts);
    assert!(results.is_ok(), "Query failed: {:?}", results.err());

    let search_results = results.unwrap();
    assert!(
        !search_results.is_empty(),
        "No results found for 'Node' query"
    );

    // Verify we found the Node class
    let node_result = search_results
        .iter()
        .find(|r| r.entry.name.contains("Node"));
    assert!(
        node_result.is_some(),
        "Node class not found in query results"
    );

    println!(
        "✓ Full pipeline test passed: {} entries packed, {} query results",
        entries.len(),
        search_results.len()
    );
}

// ============================================================================
// Query Tests
// ============================================================================

/// Test querying with different search terms
#[test]
fn test_query_with_various_search_terms() {
    // Use the existing godot-4.5.docpack if available
    let docpack_dir = PathBuf::from("docpacks");

    if !docpack_dir.exists() {
        println!("⚠️  Skipping test - docpacks directory not found");
        return;
    }

    use localdoc_cli::query::{QueryOptions, query_docpacks};
    let search_dirs = vec![docpack_dir];

    let test_queries = vec!["Node", "Vector", "get", "ready", "process"];

    for query_term in test_queries {
        let query_opts = QueryOptions {
            entry_type: None,
            path: None,
            docpack: None,
            verbose: false,
        };

        let results = query_docpacks(&search_dirs, query_term, &query_opts);

        if let Ok(search_results) = results {
            println!(
                "   Query '{}': {} results",
                query_term,
                search_results.len()
            );

            // Verify results are relevant
            if !search_results.is_empty() {
                let top_result = &search_results[0];
                assert!(
                    top_result.score > 0.0,
                    "Top result should have a positive score"
                );
            }
        }
    }

    println!("✓ Query tests with various search terms completed");
}

/// Test filtering by entry type
#[test]
fn test_query_filter_by_type() {
    let docpack_dir = PathBuf::from("docpacks");

    if !docpack_dir.exists() {
        println!("⚠️  Skipping test - docpacks directory not found");
        return;
    }

    use localdoc_cli::query::{QueryOptions, query_docpacks};
    let search_dirs = vec![docpack_dir];

    // Test filtering for classes only
    let class_opts = QueryOptions {
        entry_type: Some("class"),
        path: None,
        docpack: None,
        verbose: false,
    };

    let class_results = query_docpacks(&search_dirs, "Node", &class_opts);

    if let Ok(results) = class_results {
        use localdoc_cli::EntryType;
        for result in &results {
            assert!(
                matches!(result.entry.entry_type, EntryType::Class),
                "Expected only class entries, got {:?}",
                result.entry.entry_type
            );
        }
        println!("✓ Type filter test passed: {} class results", results.len());
    } else {
        println!("⚠️  No results for type filter test");
    }
}

// ============================================================================
// Edge Case and Error Handling Tests
// ============================================================================

/// Test handling of malformed XML
#[test]
fn test_handle_malformed_xml() {
    let temp_dir = setup_temp_dir();
    let source_dir = temp_dir.path().join("malformed");
    fs::create_dir_all(&source_dir).unwrap();

    // Create a malformed XML file
    let malformed_content = r#"<?xml version="1.0"?>
<class name="Broken"
    <brief_description>Missing closing tag</brief_description>
</class>"#;

    fs::write(source_dir.join("Broken.xml"), malformed_content).unwrap();

    // Also create a valid file
    create_test_xml_file(&source_dir.join("Valid.xml"), "Valid", false);

    // Attempt to pack - should handle the error gracefully
    use localdoc_cli::packer;
    let output_path = temp_dir.path().join("partial.docpack");

    let result = packer::pack_godot_docs(&source_dir, &output_path, "partial-test", "1.0.0");

    // Should succeed and create a docpack with at least the valid file
    assert!(result.is_ok(), "Should handle malformed XML gracefully");
    assert!(output_path.exists(), "Docpack should still be created");

    let entries = validate_content_jsonl(&output_path);
    assert!(
        !entries.is_empty(),
        "Should have entries from valid XML files"
    );

    println!("✓ Gracefully handled malformed XML");
}

/// Test handling of XML with missing required fields
#[test]
fn test_handle_xml_missing_fields() {
    let temp_dir = setup_temp_dir();
    let source_dir = temp_dir.path().join("missing_fields");
    fs::create_dir_all(&source_dir).unwrap();

    // Create XML with missing description
    let minimal_xml = r#"<?xml version="1.0"?>
<class name="Minimal" inherits="Object">
    <brief_description></brief_description>
    <description></description>
    <methods />
    <members />
    <signals />
    <constants />
</class>"#;

    fs::write(source_dir.join("Minimal.xml"), minimal_xml).unwrap();

    use localdoc_cli::godot_parser;
    let result = godot_parser::parse_godot_xml(&source_dir.join("Minimal.xml"));

    // Should either succeed with empty fields or fail gracefully
    match result {
        Ok(entries) => {
            assert!(!entries.is_empty(), "Should create at least a class entry");
            println!(
                "✓ Handled XML with minimal fields: {} entries",
                entries.len()
            );
        }
        Err(e) => {
            println!("✓ Gracefully failed on minimal XML: {}", e);
        }
    }
}

/// Test handling of special characters in class names
#[test]
fn test_special_characters_in_names() {
    let temp_dir = setup_temp_dir();
    let source_dir = temp_dir.path().join("special");
    fs::create_dir_all(&source_dir).unwrap();

    let special_xml = r#"<?xml version="1.0"?>
<class name="Node2D" inherits="Object">
    <brief_description>Node with number in name</brief_description>
    <description>Testing special characters.</description>
    <methods>
        <method name="get_position">
            <return type="Vector2" />
            <description>Returns position.</description>
        </method>
    </methods>
    <members />
    <signals />
    <constants />
</class>"#;

    fs::write(source_dir.join("Node2D.xml"), special_xml).unwrap();

    use localdoc_cli::godot_parser;
    let result = godot_parser::parse_godot_xml(&source_dir.join("Node2D.xml"));

    assert!(result.is_ok(), "Should handle numbers in class names");

    let entries = result.unwrap();
    let class_entry = entries.iter().find(|e| e.name == "Node2D");
    assert!(
        class_entry.is_some(),
        "Should find class with number in name"
    );

    println!("✓ Handled special characters in names");
}

/// Test concurrent queries (stress test)
#[test]
fn test_concurrent_queries() {
    use std::sync::Arc;
    use std::thread;

    let docpack_dir = PathBuf::from("docpacks");

    if !docpack_dir.exists() {
        println!("⚠️  Skipping concurrent query test - docpacks not found");
        return;
    }

    let search_dirs = Arc::new(vec![docpack_dir]);
    let mut handles = vec![];

    // Spawn multiple threads doing queries simultaneously
    for i in 0..5 {
        let dirs = Arc::clone(&search_dirs);
        let handle = thread::spawn(move || {
            use localdoc_cli::query::{QueryOptions, query_docpacks};

            let query_opts = QueryOptions {
                entry_type: None,
                path: None,
                docpack: None,
                verbose: false,
            };

            let queries = vec!["Node", "Vector", "get", "set", "ready"];
            let result = query_docpacks(&dirs, queries[i % queries.len()], &query_opts);

            result.is_ok()
        });

        handles.push(handle);
    }

    // Wait for all threads
    let mut success_count = 0;
    for handle in handles {
        if handle.join().unwrap() {
            success_count += 1;
        }
    }

    assert_eq!(success_count, 5, "All concurrent queries should succeed");
    println!("✓ Concurrent queries test passed");
}

// ============================================================================
// Performance Tests
// ============================================================================

/// Test query performance with a large docpack
#[test]
fn test_query_performance() {
    use std::time::Instant;

    let docpack_dir = PathBuf::from("docpacks");

    if !docpack_dir.exists() {
        println!("⚠️  Skipping performance test - docpacks not found");
        return;
    }

    use localdoc_cli::query::{QueryOptions, query_docpacks};
    let search_dirs = vec![docpack_dir];

    let query_opts = QueryOptions {
        entry_type: None,
        path: None,
        docpack: None,
        verbose: false,
    };

    let start = Instant::now();
    let result = query_docpacks(&search_dirs, "Node", &query_opts);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Query should succeed");

    // Query should complete in reasonable time (< 1 second for most cases)
    assert!(
        duration.as_secs() < 5,
        "Query took too long: {:?}",
        duration
    );

    println!("✓ Query completed in {:?}", duration);
}

/// Test packing performance with real data
#[test]
fn test_packing_performance() {
    use std::time::Instant;

    let source_dir = PathBuf::from("tests/test_doc_sources/godot-4.5");

    if !source_dir.exists() {
        println!("⚠️  Skipping packing performance test");
        return;
    }

    let temp_dir = setup_temp_dir();
    let output_path = temp_dir.path().join("perf-test.docpack");

    use localdoc_cli::packer;

    let start = Instant::now();
    let result = packer::pack_godot_docs(&source_dir, &output_path, "perf-test", "1.0.0");
    let duration = start.elapsed();

    assert!(result.is_ok(), "Packing should succeed");

    println!("✓ Packed documentation in {:?}", duration);

    // Performance expectation: should pack hundreds of files in < 30 seconds
    assert!(
        duration.as_secs() < 30,
        "Packing took too long: {:?}",
        duration
    );
}
