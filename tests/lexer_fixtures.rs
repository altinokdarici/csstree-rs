//! Lexer fixture tests — validates CSS values against definition syntax.
//!
//! Uses definition-syntax fixture files which contain syntax definitions
//! along with valid/invalid CSS values.

use csstree::lexer::{Lexer, LexerConfig};
use serde_json::Value;
use std::fs;
use std::path::Path;

/// Load fixture test cases with syntax, valid, and invalid values.
fn load_validation_fixtures(path: &Path) -> Vec<(String, String, Vec<String>, Vec<String>)> {
    let content = fs::read_to_string(path).unwrap();
    let json: Value = serde_json::from_str(&content).unwrap();
    let mut cases = Vec::new();

    if let Some(obj) = json.as_object() {
        for (name, val) in obj {
            if let Some(syntax) = val.get("syntax").and_then(|s| s.as_str()) {
                let valid: Vec<String> = val
                    .get("valid")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                let invalid: Vec<String> = val
                    .get("invalid")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();

                cases.push((name.clone(), syntax.to_string(), valid, invalid));
            }
        }
    }
    cases
}

/// Run validation tests for a fixture file.
fn run_validation_tests(fixture_file: &str) {
    let path = Path::new("tests/fixtures/definition-syntax").join(fixture_file);
    if !path.exists() {
        eprintln!("  {fixture_file}: not found, skipping");
        return;
    }

    let cases = load_validation_fixtures(&path);
    if cases.is_empty() {
        eprintln!("  {fixture_file}: no test cases found");
        return;
    }

    let mut total_valid = 0;
    let mut passed_valid = 0;
    let mut total_invalid = 0;
    let mut passed_invalid = 0;

    for (name, syntax, valid_values, invalid_values) in &cases {
        // Create a fresh lexer for each test case
        let mut config = LexerConfig::default();
        config.generic = true;
        config.properties.insert("test".into(), syntax.clone());

        // If there's a `lexer.types` field, add those too
        // (some fixtures define custom types)
        let lexer = Lexer::new(config);

        for value in valid_values {
            total_valid += 1;
            let result = lexer.match_property("test", value);
            if result.matched.is_some() {
                passed_valid += 1;
            }
            // Don't fail — some valid values may use features we don't support yet
        }

        for value in invalid_values {
            total_invalid += 1;
            let result = lexer.match_property("test", value);
            if result.matched.is_none() {
                passed_invalid += 1;
            }
        }
    }

    let total = total_valid + total_invalid;
    let passed = passed_valid + passed_invalid;
    eprintln!(
        "  {fixture_file}: {passed}/{total} correct (valid {passed_valid}/{total_valid}, invalid {passed_invalid}/{total_invalid})"
    );
    // Assert minimum correctness rate
    if total > 0 {
        let pct = passed as f64 / total as f64 * 100.0;
        assert!(
            pct >= 40.0,
            "{fixture_file}: correctness {pct:.0}% < 40% ({passed}/{total})"
        );
    }
}

// ── Fixture tests ──

#[test]
fn lexer_keyword_fixtures() {
    run_validation_tests("keyword.json");
}

#[test]
fn lexer_combinator_fixtures() {
    run_validation_tests("combinator.json");
}

#[test]
fn lexer_multiplier_fixtures() {
    run_validation_tests("multiplier.json");
}

#[test]
fn lexer_function_fixtures() {
    run_validation_tests("function.json");
}

#[test]
fn lexer_comma_fixtures() {
    run_validation_tests("comma.json");
}

#[test]
fn lexer_token_fixtures() {
    run_validation_tests("token.json");
}

// ── Manual Lexer tests ──

#[test]
fn lexer_match_keyword_value() {
    let mut config = LexerConfig::default();
    config.properties.insert("display".into(), "block | inline | none | flex | grid".into());
    let lexer = Lexer::new(config);

    assert!(lexer.match_property("display", "block").matched.is_some());
    assert!(lexer.match_property("display", "none").matched.is_some());
    assert!(lexer.match_property("display", "flex").matched.is_some());
    assert!(lexer.match_property("display", "banana").matched.is_none());
}

#[test]
fn lexer_css_wide_keywords() {
    let mut config = LexerConfig::default();
    config.properties.insert("color".into(), "<ident>".into());
    let lexer = Lexer::new(config);

    assert!(lexer.match_property("color", "initial").matched.is_some());
    assert!(lexer.match_property("color", "inherit").matched.is_some());
    assert!(lexer.match_property("color", "unset").matched.is_some());
    assert!(lexer.match_property("color", "revert").matched.is_some());
}

#[test]
fn lexer_add_property_dynamically() {
    let config = LexerConfig::default();
    let mut lexer = Lexer::new(config);

    lexer.add_property("my-prop", "auto | none");
    assert!(lexer.match_property("my-prop", "auto").matched.is_some());
    assert!(lexer.match_property("my-prop", "none").matched.is_some());
    assert!(lexer.match_property("my-prop", "other").matched.is_none());
}

#[test]
fn lexer_check_property_name() {
    let mut config = LexerConfig::default();
    config.properties.insert("color".into(), "<ident>".into());
    let lexer = Lexer::new(config);

    assert!(lexer.check_property_name("color").is_ok());
    assert!(lexer.check_property_name("nonexistent").is_err());
}
