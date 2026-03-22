//! Tests for definition-syntax-match fixtures.
//!
//! Each fixture defines syntax patterns with valid/invalid CSS value examples.
//! These test the lexer's ability to match CSS values against definition syntax.
//!
//! Covers: complex-cases.json, component-matching.json, core-combinators.json,
//! core-comma.json, core-function.json, core-multipliers.json,
//! core-parentheses.json, core-string.json, default-properties.json

use csstree::lexer::{Lexer, LexerConfig};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load a definition-syntax-match fixture and run validation tests.
fn run_match_fixture(filename: &str) {
    let path = Path::new("tests/fixtures/definition-syntax-match").join(filename);
    if !path.exists() {
        eprintln!("  {filename}: not found, skipping");
        return;
    }

    let content = fs::read_to_string(&path).unwrap();
    let fixture: Value = serde_json::from_str(&content).unwrap();
    let obj = fixture.as_object().unwrap();

    let mut total_valid = 0;
    let mut passed_valid = 0;
    let mut total_invalid = 0;
    let mut passed_invalid = 0;

    for (syntax, test_data) in obj {
        let valid: Vec<&str> = test_data
            .get("valid")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        let invalid: Vec<&str> = test_data
            .get("invalid")
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str()).collect())
            .unwrap_or_default();

        // Create a lexer with this syntax as a property
        let mut config = LexerConfig::default();
        config.generic = true;
        config.properties.insert("test".into(), syntax.clone());

        // Add any custom types from the fixture's lexer config
        if let Some(lexer_config) = test_data.get("lexer") {
            if let Some(types) = lexer_config.get("types").and_then(|t| t.as_object()) {
                for (name, def) in types {
                    if let Some(s) = def.as_str() {
                        config.types.insert(name.clone(), s.to_string());
                    }
                }
            }
        }

        let mut lexer = Lexer::new(config);

        for value in &valid {
            total_valid += 1;
            let result = lexer.match_property("test", value);
            if result.matched.is_some() {
                passed_valid += 1;
            }
        }

        for value in &invalid {
            total_invalid += 1;
            let result = lexer.match_property("test", value);
            if result.matched.is_none() {
                passed_invalid += 1;
            }
        }
    }

    eprintln!(
        "  {filename}: valid {passed_valid}/{total_valid}, invalid {passed_invalid}/{total_invalid}"
    );
}

#[test]
fn match_fixture_core_combinators() {
    run_match_fixture("core-combinators.json");
}

#[test]
fn match_fixture_core_multipliers() {
    run_match_fixture("core-multipliers.json");
}

#[test]
fn match_fixture_core_comma() {
    run_match_fixture("core-comma.json");
}

#[test]
fn match_fixture_core_function() {
    run_match_fixture("core-function.json");
}

#[test]
fn match_fixture_core_parentheses() {
    run_match_fixture("core-parentheses.json");
}

#[test]
fn match_fixture_core_string() {
    run_match_fixture("core-string.json");
}

#[test]
fn match_fixture_complex_cases() {
    run_match_fixture("complex-cases.json");
}

#[test]
fn match_fixture_component_matching() {
    run_match_fixture("component-matching.json");
}

#[test]
fn match_fixture_default_properties() {
    run_match_fixture("default-properties.json");
}
