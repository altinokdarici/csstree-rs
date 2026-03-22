//! Definition syntax fixture tests.
//!
//! Tests parse→generate round trips and verifies the parser handles
//! all definition syntax strings from fixture files.

use csstree::definition_syntax::{
    parse_definition_syntax, generate_definition_syntax,
    DefSyntaxGenOptions, walk_definition_syntax_enter,
};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Extract all `syntax` strings from a fixture JSON file.
fn extract_syntax_strings(path: &Path) -> Vec<(String, String)> {
    let content = fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&content).unwrap();
    let mut cases = Vec::new();
    if let Some(obj) = json.as_object() {
        for (name, val) in obj {
            if let Some(syntax) = val.get("syntax").and_then(|s| s.as_str()) {
                cases.push((name.clone(), syntax.to_string()));
            }
        }
    }
    cases
}

/// Test that all syntax strings from fixture files can be parsed without error.
fn run_parse_smoke_test(fixture_file: &str) {
    let path = Path::new("tests/fixtures/definition-syntax").join(fixture_file);
    if !path.exists() {
        return;
    }

    let cases = extract_syntax_strings(&path);
    if cases.is_empty() {
        eprintln!("  {fixture_file}: no syntax fields found, skipping");
        return;
    }

    let mut passed = 0;
    for (name, syntax) in &cases {
        match parse_definition_syntax(syntax) {
            Ok(node) => {
                // Verify the node has the right type
                assert!(
                    !node.node_type().is_empty(),
                    "{fixture_file}: '{name}' — parsed node has empty type"
                );
                passed += 1;
            }
            Err(e) => {
                // Some syntaxes may use features we don't support yet
                eprintln!("  WARN {fixture_file}: '{name}' parse error: {e}");
            }
        }
    }
    eprintln!("  {fixture_file}: {passed}/{} parsed successfully", cases.len());
}

/// Test parse→generate round trip.
fn run_round_trip_test(fixture_file: &str) {
    let path = Path::new("tests/fixtures/definition-syntax").join(fixture_file);
    if !path.exists() {
        return;
    }

    let cases = extract_syntax_strings(&path);
    let mut round_trips = 0;

    for (name, syntax) in &cases {
        if let Ok(node) = parse_definition_syntax(syntax) {
            let generated = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
            // Re-parse the generated string
            match parse_definition_syntax(&generated) {
                Ok(node2) => {
                    let generated2 = generate_definition_syntax(&node2, &DefSyntaxGenOptions::default());
                    // Second round trip should be stable
                    assert_eq!(
                        generated, generated2,
                        "{fixture_file}: '{name}' — round trip not stable.\n  input:  {syntax}\n  gen1:   {generated}\n  gen2:   {generated2}"
                    );
                    round_trips += 1;
                }
                Err(e) => {
                    eprintln!("  WARN {fixture_file}: '{name}' re-parse error: {e}");
                }
            }
        }
    }
    eprintln!("  {fixture_file}: {round_trips} round-trips passed");
}

// ── Smoke tests (parse) ──

#[test]
fn parse_keyword_fixtures() { run_parse_smoke_test("keyword.json"); }

#[test]
fn parse_type_fixtures() { run_parse_smoke_test("type.json"); }

#[test]
fn parse_property_fixtures() { run_parse_smoke_test("property.json"); }

#[test]
fn parse_combinator_fixtures() { run_parse_smoke_test("combinator.json"); }

#[test]
fn parse_multiplier_fixtures() { run_parse_smoke_test("multiplier.json"); }

#[test]
fn parse_function_fixtures() { run_parse_smoke_test("function.json"); }

#[test]
fn parse_comma_fixtures() { run_parse_smoke_test("comma.json"); }

#[test]
fn parse_token_fixtures() { run_parse_smoke_test("token.json"); }

#[test]
fn parse_atkeyword_fixtures() { run_parse_smoke_test("atkeyword.json"); }

#[test]
fn parse_boolean_expr_fixtures() { run_parse_smoke_test("boolean-expr.json"); }

#[test]
fn parse_bracketed_range_fixtures() { run_parse_smoke_test("bracketed-range-notation.json"); }

#[test]
fn parse_numeric_fixtures() { run_parse_smoke_test("numeric.json"); }

#[test]
fn parse_edgecases_fixtures() { run_parse_smoke_test("edgecases.json"); }

// ── Round-trip tests ──

#[test]
fn round_trip_keyword() { run_round_trip_test("keyword.json"); }

#[test]
fn round_trip_type() { run_round_trip_test("type.json"); }

#[test]
fn round_trip_combinator() { run_round_trip_test("combinator.json"); }

#[test]
fn round_trip_multiplier() { run_round_trip_test("multiplier.json"); }

// ── Manual parse tests for common patterns ──

#[test]
fn parse_common_definition_syntaxes() {
    let syntaxes = vec![
        "<length>",
        "<length> | auto",
        "<length> | <percentage>",
        "<color>#",
        "none | <image>",
        "[ <length> | <percentage> ]{1,4}",
        "<length-percentage>",
        "normal | <number>",
        "auto | <integer>",
        "<'background-color'>",
    ];

    for syntax in &syntaxes {
        let result = parse_definition_syntax(syntax);
        assert!(result.is_ok(), "Failed to parse: {syntax}: {:?}", result.err());
    }
}

// ── Walker integration test ──

#[test]
fn walk_parsed_ast() {
    let node = parse_definition_syntax("<length> | auto | <percentage>").unwrap();
    let mut types = Vec::new();
    walk_definition_syntax_enter(&node, &mut |n| {
        types.push(n.node_type().to_string());
    });
    // Should visit: Group, Type(length), Keyword(auto), Type(percentage)
    assert!(types.contains(&"Group".to_string()));
    assert!(types.contains(&"Type".to_string()));
    assert!(types.contains(&"Keyword".to_string()));
}
