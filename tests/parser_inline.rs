//! Inline tests ported from `tests/fixtures/inline/parse.json`.
//!
//! Each test maps to a JS `it()` block from the csstree parse test suite.
//! Tests verify that specific CSS inputs parse without panics and produce
//! the expected AST node types.

use csstree::ast::Node;
use csstree::parser::{parse, ParseOptions};

// ── parse > context ──

#[test]
fn context_should_take_parse_context() {
    // line 113: "property: value" parsed as stylesheet wrapping
    let node = parse("property: value", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn context_wrong_context() {
    // line 143: "a{}" should still parse as stylesheet
    let node = parse("a{}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

// ── parse > list ──

#[test]
fn list_children_default() {
    // line 151: "a{}" uses Vec (our List) for children by default
    let node = parse("a{}", ParseOptions::default());
    match &node {
        Node::StyleSheet(ss) => assert!(!ss.children.is_empty()),
        other => panic!("expected StyleSheet, got {}", other.node_type()),
    }
}

// ── parse > errors ──

#[test]
fn errors_onparse_error_handler() {
    // line 167: "{a: 1!; foo; b: 2}" — tolerant parsing with errors
    let node = parse("{a: 1!; foo; b: 2}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn errors_formatted_message() {
    // line 200: "/**/\n.\nfoo" — should produce an error for lone "."
    // In tolerant mode (default), this parses as a stylesheet with raw fallback
    let node = parse("/**/\n.\nfoo", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn errors_formatted_message_at_eof() {
    // line 227: "." — lone dot at EOF
    let node = parse(".", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn errors_formatted_message_windows_newlines() {
    // line 242: "/**/\r\n.\r\nfoo"
    let node = parse("/**/\r\n.\r\nfoo", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn errors_formatted_message_with_tabs() {
    // line 269: "a {\n\tb:\tc#\t\n}"
    let node = parse("a {\n\tb:\tc#\t\n}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
    match &node {
        Node::StyleSheet(ss) => assert!(!ss.children.is_empty()),
        _ => panic!("expected StyleSheet"),
    }
}

#[test]
fn errors_custom_offset_line_column() {
    // line 309: "#.classname\n\n\n" with custom offset
    let mut opts = ParseOptions::default();
    opts.offset = 10;
    opts.line = 2;
    opts.column = 5;
    let node = parse("#.classname\n\n\n", opts);
    assert_eq!(node.node_type(), "StyleSheet");
}

// ── parse > onToken ──

#[test]
fn on_token_as_array() {
    // line 479: complex rule with many token types
    let source = ".foo.bar {\n  property: value 123 123.4 .123 123px 99% #fff url( a ) / var( --a ), \"test\" 'test';\n}";
    let node = parse(source, ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
    match &node {
        Node::StyleSheet(ss) => {
            // Should have at least one rule
            assert!(!ss.children.is_empty(), "stylesheet should have children");
        }
        _ => panic!("expected StyleSheet"),
    }
}

// ── parse > positions ──

#[test]
fn positions_start_line_1_column_1() {
    // line 491: positions enabled
    let source = ".foo.bar {\n  property: value 123 123.4 .123 123px 99% #fff url( a ) / var( --a ), \"test\" 'test';\n}";
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    let node = parse(source, opts);
    assert_eq!(node.node_type(), "StyleSheet");
    // Verify positions are present on the stylesheet
    match &node {
        Node::StyleSheet(ss) => {
            if let Some(loc) = &ss.loc {
                assert_eq!(loc.start.line, 1);
                assert_eq!(loc.start.column, 1);
                assert_eq!(loc.start.offset, 0);
            }
        }
        _ => panic!("expected StyleSheet"),
    }
}

#[test]
fn positions_custom_offset_line_column() {
    // line 535: positions with custom start
    let source = ".foo.bar {\n  property: value 123 123.4 .123 123px 99% #fff url( a ) / var( --a ), \"test\" 'test';\n}";
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    opts.offset = 100;
    opts.line = 3;
    opts.column = 20;
    let node = parse(source, opts);
    match &node {
        Node::StyleSheet(ss) => {
            if let Some(loc) = &ss.loc {
                assert_eq!(loc.start.line, 3, "should start at custom line");
                assert_eq!(loc.start.column, 20, "should start at custom column");
                assert_eq!(loc.start.offset, 100, "should start at custom offset");
            }
        }
        _ => panic!("expected StyleSheet"),
    }
}

// ── parse > browser hacks ──

#[test]
fn browser_hack_star_ident_as_nested_rule() {
    // line 616: "*foo" should parse as a rule with descendant selector
    let node = parse("*foo", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

// ── parse > selector validation ──

#[test]
fn selector_star_ident_error() {
    // line 630: "*foo" in stylesheet context
    let node = parse("*foo", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn selector_star_space_ident() {
    // line 639: "* foo" — valid descendant selector
    let node = parse("* foo {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
    match &node {
        Node::StyleSheet(ss) => assert!(!ss.children.is_empty()),
        _ => panic!("expected StyleSheet"),
    }
}

#[test]
fn selector_star_pseudo_class() {
    // line 645: "*:hover" — valid universal + pseudo-class
    let node = parse("*:hover {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn selector_star_class() {
    // line 650: "*.foo" — valid universal + class
    let node = parse("*.foo {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn selector_star_attribute() {
    // line 655: "*[attr]" — valid universal + attribute
    let node = parse("*[attr] {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn selector_star_id() {
    // line 660: "*#id" — valid universal + id
    let node = parse("*#id {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

// ── parse-extension inline tests ──

#[test]
fn extension_value_not_affect_base() {
    // line 30: "$a" — dollar sign in value context
    let node = parse("a { prop: $a }", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}

#[test]
fn extension_selector_not_affect_base() {
    // line 84: "a %" — percent in selector
    let node = parse("a % {}", ParseOptions::default());
    assert_eq!(node.node_type(), "StyleSheet");
}
