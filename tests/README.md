# LocalDoc CLI Testing Documentation

## Overview

This directory contains comprehensive integration tests for the localdoc CLI tool. The tests are designed to validate the entire pipeline from parsing raw documentation sources to packing them into docpacks and querying them.

## Test Structure

### Test Categories

1. **Basic Validation Tests**
   - Manifest creation and parsing
   - Version constant validation
   - Directory structure validation

2. **XML Parsing Edge Cases**
   - Valid Godot XML parsing
   - Multiple file parsing
   - Malformed XML handling
   - Missing required fields
   - Special characters in names
   - Unicode and emoji support
   - Nested markup handling

3. **Packing Tests**
   - Simple docpack creation
   - Real Godot documentation packing
   - Empty directory handling
   - Partial success with errors

4. **End-to-End Pipeline Tests**
   - Full workflow: build → pack → query
   - Result validation at each stage

5. **Query Tests**
   - Various search terms
   - Type filtering (class, method, etc.)
   - Docpack-specific queries

6. **Performance Tests**
   - Query performance benchmarking
   - Packing performance with large datasets
   - Concurrent query stress tests

## Test Data Sources

### Real Documentation: `tests/test_doc_sources/godot-4.5/`

Contains actual Godot 4.5 XML class documentation files. These are real-world examples used to ensure the parser can handle production-quality documentation.

Key files for testing:
- `Node.xml` - Core scene tree class with extensive documentation
- `Node2D.xml` - 2D scene node with spatial properties
- `Object.xml` - Base class for all Godot classes
- `Vector2.xml`, `Vector3.xml` - Mathematical types
- Various other classes covering different API surfaces

### Edge Cases: `tests/test_doc_sources/edge_cases/`

Contains synthetic test files designed to test edge cases and error handling:

- **`Malformed.xml`** - Intentionally broken XML to test error handling
- **`Empty.xml`** - Minimal valid XML with empty fields
- **`SpecialChars.xml`** - Special characters, Unicode, emoji, mathematical symbols
- **`VeryLong.xml`** - Large file with many methods and long descriptions
- **`NoInherits.xml`** - Class without inheritance (tests optional fields)
- **`NestedMarkup.xml`** - Rich BBCode-style markup in descriptions

## Running Tests

### Run All Tests
```bash
cargo test
```

### Run Specific Test Categories
```bash
# Run only integration tests
cargo test --test integration_test

# Run tests with output
cargo test -- --nocapture

# Run specific test by name
cargo test test_full_pipeline_build_pack_query -- --nocapture
```

### Run Tests in Parallel
```bash
# Default behavior (parallel)
cargo test

# Force serial execution (useful for debugging)
cargo test -- --test-threads=1
```

## Test Helpers

The test suite includes several helper functions:

### `setup_temp_dir() -> TempDir`
Creates a temporary directory that is automatically cleaned up after the test.

### `get_cli_binary() -> PathBuf`
Locates the compiled CLI binary for integration testing.

### `run_cli(args: &[&str]) -> Output`
Executes the CLI with given arguments and returns the output.

### `create_test_xml_file(path: &Path, class_name: &str, has_methods: bool)`
Creates a minimal valid Godot XML file for testing.

### `validate_manifest(docpack_dir: &Path) -> serde_json::Value`
Validates that a docpack has a well-formed manifest.json file.

### `validate_content_jsonl(docpack_dir: &Path) -> Vec<serde_json::Value>`
Validates that a docpack has a well-formed content.jsonl file.

## Expected Behavior

### Successful Tests
- All parsing tests should succeed for valid XML files
- Malformed XML should be handled gracefully (warnings, not crashes)
- Empty directories should create valid (but empty) docpacks
- Queries should return relevant results with positive scores
- Performance tests should complete within reasonable time limits

### Error Handling
- Malformed XML: Parser should log a warning and continue with other files
- Missing files: Should return appropriate error messages
- Empty queries: Should return empty result sets, not errors
- Concurrent access: Should handle multiple simultaneous queries safely

## Performance Expectations

Based on test data:
- **Parsing**: ~100-500 XML files per second
- **Packing**: Full Godot docs (1000+ files) in < 30 seconds
- **Querying**: Single query response in < 1 second
- **Concurrent queries**: 5+ simultaneous queries without degradation

## Adding New Tests

When adding new tests:

1. **Use descriptive names**: `test_<category>_<specific_case>`
2. **Add documentation**: Explain what the test validates
3. **Use helper functions**: Reuse existing setup/validation code
4. **Test edge cases**: Consider boundary conditions
5. **Include assertions**: Verify both success and failure cases
6. **Clean up**: Use `TempDir` for temporary files

Example:
```rust
/// Test handling of XML files with duplicate method names
#[test]
fn test_parse_duplicate_method_names() {
    let temp_dir = setup_temp_dir();
    // ... test implementation
    println!("✓ Duplicate method test passed");
}
```

## Continuous Integration

These tests are designed to run in CI environments:
- No external dependencies required
- Self-contained test data
- Reasonable execution time (< 5 minutes total)
- Clear pass/fail indicators

## Troubleshooting

### Test Fails: "docpacks directory not found"
This is expected if you haven't run `cargo run -- pack` yet. The test will skip gracefully.

### Test Fails: "Failed to parse X.xml"
Check that the test data files exist and haven't been corrupted. Re-clone if necessary.

### Performance Tests Fail
These have generous time limits but may fail on slow systems. Consider:
- Running with `--release` flag for optimized builds
- Increasing timeout thresholds for your environment
- Checking for background processes consuming resources

### Concurrent Test Failures
If concurrent tests fail intermittently:
- Run with `--test-threads=1` to isolate the issue
- Check for file system race conditions
- Verify proper use of `TempDir` for isolation

## Future Enhancements

Potential additions to the test suite:
- [ ] Fuzzing tests for parser robustness
- [ ] Memory leak detection
- [ ] Cross-platform path handling tests
- [ ] Network-related tests (if applicable)
- [ ] Regression tests for specific bug fixes
- [ ] Property-based testing with quickcheck
- [ ] Code coverage measurement
- [ ] Benchmark comparisons over time

## Contributing

When contributing tests:
1. Ensure all existing tests pass
2. Add tests for new features
3. Document expected behavior
4. Include both positive and negative test cases
5. Update this README if adding new test categories
