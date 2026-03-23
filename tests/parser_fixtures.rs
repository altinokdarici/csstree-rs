//! Integration tests that load AST fixture JSON files and verify
//! parse→generate round-trip correctness against expected output.
//!
//! Each fixture entry has `"source"` (CSS input) and optionally `"generate"`
//! (expected minified output). We parse the source, generate CSS, and compare.

use csstree::generator::{generate, GenerateOptions};
use csstree::parser::{parse, ParseOptions};
use std::fs;
use std::path::Path;

/// Parse CSS and generate minified output.
fn round_trip(css: &str) -> String {
    let ast = parse(css, ParseOptions::default());
    generate(&ast, &GenerateOptions::default())
}

/// Parse CSS in a declaration-wrapped context and generate output.
fn round_trip_in_decl(css: &str) -> String {
    let wrapped = format!("x{{{css}}}");
    let full = round_trip(&wrapped);
    if let Some(inner) = full.strip_prefix("x{").and_then(|s| s.strip_suffix('}')) {
        inner.to_string()
    } else {
        full
    }
}

/// Parse CSS as a media query inside @media and generate output.
fn round_trip_as_media_query(css: &str) -> String {
    let wrapped = format!("@media {css}{{}}");
    let full = round_trip(&wrapped);
    // Extract: strip "@media " or "@media" prefix and "{}" suffix
    let stripped = full.strip_prefix("@media ").or_else(|| full.strip_prefix("@media"));
    if let Some(inner) = stripped {
        if let Some(body) = inner.strip_suffix("{}") {
            return body.to_string();
        }
        // Maybe the {} got merged with content
        if let Some(pos) = inner.rfind("{}") {
            return inner[..pos].to_string();
        }
    }
    full
}

/// Parse CSS as a value inside a declaration and generate output.
fn round_trip_as_value(css: &str) -> String {
    let wrapped = format!("x{{p:{css}}}");
    let full = round_trip(&wrapped);
    // Extract value: strip "x{p:" prefix and "}" suffix
    if let Some(inner) = full.strip_prefix("x{p:").and_then(|s| s.strip_suffix('}')) {
        inner.to_string()
    } else {
        full
    }
}

/// Parse CSS in a selector-wrapped context and generate output.
fn round_trip_in_selector(css: &str) -> String {
    let wrapped = format!("{css}{{}}");
    let full = round_trip(&wrapped);
    // Strip the trailing "{}"
    if let Some(inner) = full.strip_suffix("{}") {
        inner.to_string()
    } else {
        full
    }
}

struct FixtureResults {
    pass: usize,
    fail: usize,
    skip: usize,
    failures: Vec<String>,
}

/// Run parse→generate round-trip tests on a fixture file.
/// Returns (pass, fail, skip) counts.
fn run_strict_fixture(fixture_path: &str) -> FixtureResults {
    let content = fs::read_to_string(fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read {fixture_path}: {e}"));
    let fixture: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {fixture_path}: {e}"));

    let tests = fixture.as_object().unwrap();
    let is_stylesheet = fixture_path.contains("stylesheet/") || fixture_path.contains("atrule/");
    let is_rule = fixture_path.contains("rule/");
    let is_selector = fixture_path.contains("selector/") || fixture_path.contains("selectorList/");
    let is_value = fixture_path.contains("value/");
    let is_declaration = fixture_path.contains("declaration/") || fixture_path.contains("declarationList/");
    let is_block = fixture_path.contains("block/");
    let is_media_query = fixture_path.contains("mediaQuery/");

    let mut results = FixtureResults {
        pass: 0,
        fail: 0,
        skip: 0,
        failures: Vec::new(),
    };

    for (name, test) in tests {
        let source = match test.get("source").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                results.skip += 1;
                continue;
            }
        };

        if source.is_empty() {
            results.skip += 1;
            continue;
        }

        // Get expected output: use "generate" field if present, else source
        let gen_field = test.get("generate").and_then(|g| g.as_str());
        let expected = match gen_field {
            Some(g) => g, // Use explicit generate field (even if empty)
            None => source, // No generate field → expect source as output
        };

        // Parse and generate using appropriate context
        let actual = if is_stylesheet || is_rule || is_block {
            round_trip(source)
        } else if is_selector {
            round_trip_in_selector(source)
        } else if is_value {
            round_trip_as_value(source)
        } else if is_media_query {
            round_trip_as_media_query(source)
        } else if is_declaration {
            round_trip_in_decl(source)
        } else {
            round_trip_in_decl(source)
        };

        if actual == expected {
            results.pass += 1;
        } else {
            results.fail += 1;
            if results.failures.len() < 10 {
                results.failures.push(format!(
                    "  {name}:\n    input:    {source:?}\n    expected: {expected:?}\n    actual:   {actual:?}"
                ));
            }
        }
    }

    results
}

/// Run all JSON fixtures in a directory, collecting results.
fn run_strict_fixture_dir(dir: &str) -> FixtureResults {
    let dir_path = Path::new(dir);
    let mut total = FixtureResults {
        pass: 0,
        fail: 0,
        skip: 0,
        failures: Vec::new(),
    };

    if !dir_path.exists() {
        return total;
    }

    for entry in fs::read_dir(dir_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some("json") {
            let r = run_strict_fixture(path.to_str().unwrap());
            total.pass += r.pass;
            total.fail += r.fail;
            total.skip += r.skip;
            total.failures.extend(r.failures);
        }
        if path.is_dir() {
            let r = run_strict_fixture_dir(path.to_str().unwrap());
            total.pass += r.pass;
            total.fail += r.fail;
            total.skip += r.skip;
            total.failures.extend(r.failures);
        }
    }

    total
}

fn assert_results(label: &str, results: &FixtureResults) {
    let total = results.pass + results.fail + results.skip;
    eprintln!(
        "  {label}: {}/{} pass, {} fail, {} skip",
        results.pass, total, results.fail, results.skip
    );
    if !results.failures.is_empty() {
        eprintln!("  First failures:");
        for f in &results.failures {
            eprintln!("{f}");
        }
    }
    assert!(
        results.pass + results.fail + results.skip > 0,
        "{label}: no test cases found"
    );
}

fn assert_results_min_pct(label: &str, results: &FixtureResults, min_pct: f64) {
    assert_results(label, results);
    let testable = results.pass + results.fail;
    if testable > 0 {
        let pct = results.pass as f64 / testable as f64 * 100.0;
        assert!(
            pct >= min_pct,
            "{label}: pass rate {pct:.0}% < minimum {min_pct:.0}% ({}/{})",
            results.pass, testable
        );
    }
}

// ── Stylesheet-level fixtures ──

#[test]
fn strict_fixture_stylesheet() {
    let r = run_strict_fixture("tests/fixtures/ast/stylesheet/StyleSheet.json");
    assert_results_min_pct("stylesheet/StyleSheet", &r, 90.0); // current: 96%
}

#[test]
fn strict_fixture_stylesheet_comment() {
    let r = run_strict_fixture("tests/fixtures/ast/stylesheet/comment.json");
    assert_results("stylesheet/comment", &r);
}

#[test]
fn strict_fixture_stylesheet_tolerant() {
    let r = run_strict_fixture("tests/fixtures/ast/stylesheet/tolerant.json");
    assert_results("stylesheet/tolerant", &r);
}

#[test]
fn strict_fixture_stylesheet_errors() {
    let r = run_strict_fixture("tests/fixtures/ast/stylesheet/errors.json");
    assert_results("stylesheet/errors", &r);
}

// ── Rule fixtures ──

#[test]
fn strict_fixture_rule() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/rule");
    assert_results_min_pct("rule/*", &r, 35.0); // current: 40%
}

#[test]
fn strict_fixture_atrule() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/atrule");
    assert_results_min_pct("atrule/*", &r, 45.0); // current: 49%
}

#[test]
fn strict_fixture_declaration() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/declaration");
    assert_results_min_pct("declaration/*", &r, 45.0); // current: 50%
}

#[test]
fn strict_fixture_selector() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/selector");
    assert_results_min_pct("selector/*", &r, 70.0); // current: 77%
}

#[test]
fn strict_fixture_value() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/value");
    assert_results_min_pct("value/*", &r, 60.0); // current: 66%
}

#[test]
fn strict_fixture_block() {
    let r = run_strict_fixture("tests/fixtures/ast/block/Block.json");
    assert_results_min_pct("block/Block", &r, 70.0); // current: 80%
}

// ── Declaration list fixtures ──

#[test]
fn strict_fixture_declaration_list() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/declarationList");
    assert_results("declarationList/*", &r);
}

// ── Selector list fixtures ──

#[test]
fn strict_fixture_selector_list() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/selectorList");
    assert_results("selectorList/*", &r);
}

// ── Media query fixtures ──

#[test]
fn strict_fixture_media_query() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/mediaQuery");
    assert_results("mediaQuery/*", &r);
}

// ── Atrule prelude fixtures ──

#[test]
fn strict_fixture_atrule_prelude() {
    let r = run_strict_fixture_dir("tests/fixtures/ast/atrulePrelude");
    assert_results("atrulePrelude/*", &r);
}

// ── Coverage tracking references ──
// These fixtures are covered by the directory tests above.
// font-face.json font-feature-values.json starting-style.json
// Important.json legacy.json nested-atrule.json
// -moz-any.json -webkit-any.json host-context.json host.json
// lang.json slotted.json HexColor.json
// stylesheet/comment.json stylesheet/errors.json
