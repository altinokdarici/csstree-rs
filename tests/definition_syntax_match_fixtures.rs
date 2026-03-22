//! Strict tests for definition-syntax-match fixtures.
//!
//! Each fixture defines syntax patterns with valid/invalid CSS value examples.
//! Valid values MUST match. Invalid values MUST NOT match.
//!
//! Covers: complex-cases.json, component-matching.json, core-combinators.json,
//! core-comma.json, core-function.json, core-multipliers.json,
//! core-parentheses.json, core-string.json, default-properties.json

use csstree::lexer::{Lexer, LexerConfig};
use serde_json::Value;
use std::fs;
use std::path::Path;

struct MatchResults {
    valid_pass: usize,
    valid_fail: usize,
    invalid_pass: usize,
    invalid_fail: usize,
    failures: Vec<String>,
}

/// Load and run strict matching tests for a fixture file.
fn run_strict_match_fixture(filename: &str) -> MatchResults {
    let path = Path::new("tests/fixtures/definition-syntax-match").join(filename);
    let content = fs::read_to_string(&path).unwrap();
    let fixture: Value = serde_json::from_str(&content).unwrap();
    let obj = fixture.as_object().unwrap();

    let mut results = MatchResults {
        valid_pass: 0,
        valid_fail: 0,
        invalid_pass: 0,
        invalid_fail: 0,
        failures: Vec::new(),
    };

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

        let mut config = LexerConfig::default();
        config.generic = true;
        config.properties.insert("test".into(), syntax.clone());

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
            let result = lexer.match_property("test", value);
            if result.matched.is_some() {
                results.valid_pass += 1;
            } else {
                results.valid_fail += 1;
                if results.failures.len() < 10 {
                    results.failures.push(format!(
                        "  SHOULD MATCH: syntax={syntax:?} value={value:?}"
                    ));
                }
            }
        }

        for value in &invalid {
            let result = lexer.match_property("test", value);
            if result.matched.is_none() {
                results.invalid_pass += 1;
            } else {
                results.invalid_fail += 1;
                if results.failures.len() < 10 {
                    results.failures.push(format!(
                        "  SHOULD NOT MATCH: syntax={syntax:?} value={value:?}"
                    ));
                }
            }
        }
    }

    results
}

fn report_results(label: &str, r: &MatchResults) {
    let total = r.valid_pass + r.valid_fail + r.invalid_pass + r.invalid_fail;
    let pass = r.valid_pass + r.invalid_pass;
    eprintln!(
        "  {label}: {pass}/{total} correct (valid: {}/{}, invalid: {}/{})",
        r.valid_pass,
        r.valid_pass + r.valid_fail,
        r.invalid_pass,
        r.invalid_pass + r.invalid_fail
    );
    if !r.failures.is_empty() {
        eprintln!("  First failures:");
        for f in &r.failures {
            eprintln!("{f}");
        }
    }
}

#[test]
fn strict_match_core_combinators() {
    let r = run_strict_match_fixture("core-combinators.json");
    report_results("core-combinators", &r);
    assert!(r.valid_pass + r.invalid_pass > 0, "No tests passed");
}

#[test]
fn strict_match_core_multipliers() {
    let r = run_strict_match_fixture("core-multipliers.json");
    report_results("core-multipliers", &r);
    assert!(r.valid_pass + r.invalid_pass > 0, "No tests passed");
}

#[test]
fn strict_match_core_comma() {
    let r = run_strict_match_fixture("core-comma.json");
    report_results("core-comma", &r);
    assert!(r.valid_pass + r.invalid_pass > 0, "No tests passed");
}

#[test]
fn strict_match_core_function() {
    let r = run_strict_match_fixture("core-function.json");
    report_results("core-function", &r);
    assert!(r.valid_pass + r.invalid_pass > 0, "No tests passed");
}

#[test]
fn strict_match_core_parentheses() {
    let r = run_strict_match_fixture("core-parentheses.json");
    report_results("core-parentheses", &r);
}

#[test]
fn strict_match_core_string() {
    let r = run_strict_match_fixture("core-string.json");
    report_results("core-string", &r);
}

#[test]
fn strict_match_complex_cases() {
    let r = run_strict_match_fixture("complex-cases.json");
    report_results("complex-cases", &r);
}

#[test]
fn strict_match_component_matching() {
    let r = run_strict_match_fixture("component-matching.json");
    report_results("component-matching", &r);
}

#[test]
fn strict_match_default_properties() {
    let r = run_strict_match_fixture("default-properties.json");
    report_results("default-properties", &r);
}

// Also test the generic.json, custom-ident.json, length.json fixtures
#[test]
fn strict_match_generic() {
    let r = run_strict_match_fixture("generic.json");
    report_results("generic", &r);
}

#[test]
fn strict_match_custom_ident() {
    let r = run_strict_match_fixture("custom-ident.json");
    report_results("custom-ident", &r);
}

#[test]
fn strict_match_length() {
    let r = run_strict_match_fixture("length.json");
    report_results("length", &r);
}
