//! Tests for source map generation, ported from generate.js sourceMap tests.

use csstree::parser::{parse, ParseOptions};
use csstree::parser::options::ParseFlags;
use csstree::generator::{generate_with_source_map, source_map_to_json, GenerateOptions};

#[test]
fn source_map_simple() {
    // generate.js: "should generate a map"
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        filename: Some("test.css".to_string()),
        ..ParseOptions::default()
    };
    let ast = parse(".a {\n  color: red;\n}\n", opts);
    let result = generate_with_source_map(&ast, &GenerateOptions::default());

    assert_eq!(result.css, ".a{color:red}");
    assert!(!result.mappings.is_empty(), "Should have source mappings");

    let json = source_map_to_json("test.css", &result.mappings);
    assert!(json.contains("\"version\":3"));
    assert!(json.contains("\"sources\":[\"test.css\"]"));
    assert!(json.contains("\"mappings\":"));
}

#[test]
fn source_map_complex() {
    // generate.js: "complex CSS"
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        filename: Some("test.css".to_string()),
        ..ParseOptions::default()
    };
    let ast = parse(
        ".a { color: #ff0000; } .b { display: block; float: left; } @media foo { .c { color: red } }",
        opts,
    );
    let result = generate_with_source_map(&ast, &GenerateOptions::default());

    assert_eq!(result.css, ".a{color:#ff0000}.b{display:block;float:left}@media foo{.c{color:red}}");
    assert!(result.mappings.len() >= 3, "Should have mappings for rules/declarations");
}

#[test]
fn source_map_mappings_have_positions() {
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        filename: Some("src.css".to_string()),
        ..ParseOptions::default()
    };
    let ast = parse(".a { color: red }", opts);
    let result = generate_with_source_map(&ast, &GenerateOptions::default());

    // Should have at least a mapping for the rule and the declaration
    assert!(result.mappings.len() >= 2);

    // First mapping should be for the rule
    let first = &result.mappings[0];
    assert_eq!(first.source, "src.css");
    assert_eq!(first.generated_line, 1);
    assert_eq!(first.generated_column, 0);
}

#[test]
fn source_map_json_format() {
    let opts = ParseOptions {
        flags: ParseFlags { positions: true, ..ParseFlags::default() },
        filename: Some("test.css".to_string()),
        ..ParseOptions::default()
    };
    let ast = parse("a { b: c }", opts);
    let result = generate_with_source_map(&ast, &GenerateOptions::default());
    let json = source_map_to_json("test.css", &result.mappings);

    // Should be valid JSON structure
    assert!(json.starts_with('{'));
    assert!(json.ends_with('}'));
    assert!(json.contains("\"version\":3"));
    assert!(json.contains("\"sources\":[\"test.css\"]"));
    assert!(json.contains("\"names\":[]"));
    assert!(json.contains("\"mappings\":"));
}
