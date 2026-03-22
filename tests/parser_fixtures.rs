//! Integration tests that load AST fixture JSON files and verify
//! the Rust parser can handle every CSS source without crashing or hanging.
//!
//! Tests wrap each CSS source in a minimal stylesheet context to parse.

use csstree::parser::{parse, ParseOptions};
use std::fs;
use std::path::Path;

/// Parse every CSS source in a fixture file as a stylesheet (wrapping in `a { ... }` if needed).
fn run_fixture(fixture_path: &str) {
    let content = fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {fixture_path}: {e}"));
    let fixture: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {fixture_path}: {e}"));

    let tests = fixture.as_object().unwrap();
    let mut pass_count = 0;

    // Determine context from fixture path
    let is_stylesheet = fixture_path.contains("stylesheet/") || fixture_path.contains("atrule/");
    let is_rule = fixture_path.contains("rule/");
    let is_block = fixture_path.contains("block/");

    for (name, test) in tests {
        let sources = extract_sources(test);
        for source in &sources {
            if source.is_empty() {
                continue;
            }

            // Wrap source in appropriate context so it parses as a stylesheet
            let wrapped = if is_stylesheet || is_rule || is_block {
                source.to_string()
            } else {
                // Selectors, values, declarations, etc. — wrap in a rule
                format!("x {{ {source} }}")
            };

            let node = parse(&wrapped, ParseOptions::default());
            assert_eq!(
                node.node_type(), "StyleSheet",
                "{fixture_path} :: {name}: source={source:?} wrapped={wrapped:?}"
            );
            pass_count += 1;
        }
    }

    if pass_count == 0 {
        // Some fixtures only have error tests or non-source entries — that's OK
        eprintln!("WARNING: {fixture_path}: 0 test cases had source");
    }
}

/// Extract source strings from a fixture test entry.
fn extract_sources(test: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = test.as_array() {
        arr.iter()
            .filter_map(|t| t.get("source").and_then(|s| s.as_str()).map(String::from))
            .collect()
    } else if let Some(s) = test.get("source").and_then(|s| s.as_str()) {
        vec![s.to_string()]
    } else {
        vec![]
    }
}

/// Run every JSON fixture file in a directory (recursive).
fn run_fixture_dir(dir: &str) {
    let dir_path = Path::new(dir);
    if !dir_path.exists() {
        return;
    }
    for entry in fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            run_fixture(path.to_str().unwrap());
        }
        if path.is_dir() {
            run_fixture_dir(path.to_str().unwrap());
        }
    }
}

// ── Stylesheet-level fixtures (parse directly) ──

#[test]
fn fixture_stylesheet() {
    run_fixture("tests/fixtures/ast/stylesheet/StyleSheet.json");
}

#[test]
fn fixture_stylesheet_tolerant() {
    run_fixture("tests/fixtures/ast/stylesheet/tolerant.json");
}

#[test]
fn fixture_rule() {
    run_fixture_dir("tests/fixtures/ast/rule");
}

#[test]
fn fixture_atrule() {
    run_fixture_dir("tests/fixtures/ast/atrule");
}

// ── Wrapped fixtures (CSS fragments wrapped in `x { ... }`) ──

#[test]
fn fixture_declaration() {
    run_fixture_dir("tests/fixtures/ast/declaration");
}

#[test]
fn fixture_selector() {
    run_fixture_dir("tests/fixtures/ast/selector");
}

#[test]
fn fixture_block() {
    run_fixture("tests/fixtures/ast/block/Block.json");
}

#[test]
fn fixture_value() {
    run_fixture_dir("tests/fixtures/ast/value");
}

#[test]
fn fixture_declaration_list() {
    run_fixture_dir("tests/fixtures/ast/declarationList");
}

#[test]
fn fixture_selector_list() {
    run_fixture_dir("tests/fixtures/ast/selectorList");
}

#[test]
fn fixture_media_query() {
    run_fixture_dir("tests/fixtures/ast/mediaQuery");
}

#[test]
fn fixture_atrule_prelude() {
    run_fixture_dir("tests/fixtures/ast/atrulePrelude");
}
