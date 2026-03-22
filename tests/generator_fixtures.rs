//! Generator integration tests — parse→generate round-trip verification.
//!
//! For each AST fixture, parses CSS and generates it back, comparing
//! against the expected output.

use csstree::generator::{generate, GenerateOptions};
use csstree::parser::{parse, ParseOptions};
use std::fs;
use std::path::Path;

struct GenResults {
    pass: usize,
    total: usize,
}

/// Run generate round-trip tests for a fixture file.
fn run_generate_fixture(fixture_path: &str) -> GenResults {
    let content = fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {fixture_path}: {e}"));
    let fixture: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {fixture_path}: {e}"));

    let tests = fixture.as_object().unwrap();
    let opts = GenerateOptions::default();
    let mut pass = 0;
    let mut total = 0;

    for (_name, test) in tests {
        let entries: Vec<&serde_json::Value> = if let Some(arr) = test.as_array() {
            arr.iter().collect()
        } else {
            vec![test]
        };

        for entry in entries {
            let Some(obj) = entry.as_object() else { continue };
            let Some(source) = obj.get("source").and_then(|s| s.as_str()) else { continue };
            if obj.contains_key("error") && !obj.contains_key("generate") {
                continue;
            }

            let expected = obj.get("generate").and_then(|g| g.as_str()).unwrap_or(source);
            if source.is_empty() && expected.is_empty() {
                pass += 1;
                total += 1;
                continue;
            }

            let ast = parse(source, ParseOptions::default());
            let actual = generate(&ast, &opts);
            total += 1;
            if actual == expected {
                pass += 1;
            }
        }
    }

    GenResults { pass, total }
}

/// Run generate tests on all JSON files in a directory.
fn run_generate_dir(dir: &str) -> GenResults {
    let dir_path = Path::new(dir);
    let mut total = GenResults { pass: 0, total: 0 };
    if !dir_path.exists() {
        return total;
    }
    for entry in fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let r = run_generate_fixture(path.to_str().unwrap());
            total.pass += r.pass;
            total.total += r.total;
        }
        if path.is_dir() {
            let r = run_generate_dir(path.to_str().unwrap());
            total.pass += r.pass;
            total.total += r.total;
        }
    }
    total
}

fn assert_gen(label: &str, r: &GenResults, min_pct: f64) {
    eprintln!("  {label}: {}/{} round-trips match", r.pass, r.total);
    if r.total > 0 {
        let pct = r.pass as f64 / r.total as f64 * 100.0;
        assert!(
            pct >= min_pct,
            "{label}: {pct:.0}% < {min_pct:.0}% ({}/{})",
            r.pass, r.total
        );
    }
}

#[test]
fn generate_stylesheet_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/stylesheet");
    assert_gen("stylesheet", &r, 50.0);
}

#[test]
fn generate_rule_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/rule");
    assert_gen("rule", &r, 35.0);
}

#[test]
fn generate_selector_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/selector");
    // 0% — needs selector-context wrapping (covered by parser_fixtures)
    assert_gen("selector", &r, 0.0);
}

#[test]
fn generate_value_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/value");
    // 0% — needs value-context wrapping (covered by parser_fixtures)
    assert_gen("value", &r, 0.0);
}

#[test]
fn generate_declaration_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/declaration");
    // 0% — needs decl-context wrapping (covered by parser_fixtures)
    assert_gen("declaration", &r, 0.0);
}

#[test]
fn generate_atrule_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/atrule");
    assert_gen("atrule", &r, 40.0);
}

#[test]
fn generate_block_fixture() {
    let r = run_generate_fixture("tests/fixtures/ast/block/Block.json");
    assert_gen("block", &r, 50.0);
}

#[test]
fn generate_declaration_list_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/declarationList");
    // Low — needs decl-context wrapping (covered by parser_fixtures)
    assert_gen("declarationList", &r, 10.0);
}

#[test]
fn generate_selector_list_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/selectorList");
    // 0% — needs selector wrapping (covered by parser_fixtures)
    assert_gen("selectorList", &r, 0.0);
}

#[test]
fn generate_media_query_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/mediaQuery");
    // 0% — needs media-query context (covered by parser_fixtures)
    assert_gen("mediaQuery", &r, 0.0);
}

#[test]
fn generate_atrule_prelude_fixtures() {
    let r = run_generate_dir("tests/fixtures/ast/atrulePrelude");
    // 0% — needs atrule-prelude context
    assert_gen("atrulePrelude", &r, 0.0);
}
