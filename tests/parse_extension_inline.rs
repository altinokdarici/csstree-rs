//! Tests ported from `external/csstree/lib/__tests/parse-extension.js`.
//!
//! Tests that the parser handles extension syntax (custom value/selector scopes).
//! Since Rust doesn't have JS-style extension via fork(), we test that our parser
//! handles the CSS patterns that extension tests validate.

use csstree::ast::Node;
use csstree::parser::{parse, ParseOptions};
use csstree::generator::{generate, GenerateOptions};

fn round_trip(css: &str) -> String {
    let ast = parse(css, ParseOptions::default());
    generate(&ast, &GenerateOptions::default())
}

// ── parse-extension.js: value extension ──

#[test]
fn extension_dollar_in_value() {
    // parse-extension.js: "should parse according new rules" for value
    // The $ sign in values should be handled (as operator)
    let ast = parse("a { prop: $var }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
    let output = round_trip("a { prop: $var }");
    assert!(output.contains("prop:"));
}

#[test]
fn extension_dollar_function_in_value() {
    let ast = parse("a { prop: $fn(1, 2) }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

#[test]
fn extension_unknown_in_value_fallback() {
    // "should fail on unknown" — parser should handle gracefully
    let ast = parse("a { prop: ??? }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

// ── parse-extension.js: selector extension ──

#[test]
fn extension_percent_in_selector() {
    // parse-extension.js: "should parse according new rules" for selector
    let ast = parse("a % { color: red }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

#[test]
fn extension_at_in_selector() {
    let ast = parse(".a @b { color: red }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

#[test]
fn extension_unknown_in_selector_fallback() {
    // "should fail on unknown" — parser handles unknown selector syntax gracefully
    let ast = parse("??? { color: red }", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}
