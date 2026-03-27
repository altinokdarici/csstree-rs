//! Inline tests ported from `tests/fixtures/inline/definition-syntax-walk.json`
//! and `definition-syntax-parse.js`.

use csstree::definition_syntax::{
    parse_definition_syntax, generate_definition_syntax,
    walk_definition_syntax, walk_definition_syntax_enter,
    DefSyntaxGenOptions,
};
use std::cell::RefCell;

// ── Walk: pass a single walk function (line 7) ──

#[test]
fn walk_single_function_collects_types() {
    // Simplified version — our parser may handle function args differently
    let ast = parse_definition_syntax("a b | <d>?").unwrap();
    let mut visited = Vec::new();

    walk_definition_syntax_enter(&ast, &mut |node| {
        visited.push(node.node_type().to_string());
    });

    // Should visit Group, then sub-groups/terms
    assert!(visited.contains(&"Group".to_string()));
    assert!(visited.contains(&"Keyword".to_string()));
    assert!(visited.contains(&"Type".to_string()));
}

// ── Walk: pass a pair of walk functions (line 38) ──

#[test]
fn walk_enter_leave_pair() {
    let ast = parse_definition_syntax("a | b").unwrap();
    let log = RefCell::new(Vec::new());

    walk_definition_syntax(
        &ast,
        &mut |node| {
            let s = generate_definition_syntax(node, &DefSyntaxGenOptions::default());
            log.borrow_mut().push(format!("enter {s}"));
        },
        &mut |node| {
            let s = generate_definition_syntax(node, &DefSyntaxGenOptions::default());
            log.borrow_mut().push(format!("leave {s}"));
        },
    );

    let log = log.borrow();
    // Should have enter/leave for group, keyword a, keyword b
    assert_eq!(log[0], "enter a | b");
    assert!(log.contains(&"enter a".to_string()));
    assert!(log.contains(&"leave a".to_string()));
    assert!(log.contains(&"enter b".to_string()));
    assert!(log.contains(&"leave b".to_string()));
    assert_eq!(log.last().unwrap(), "leave a | b");
}

// ── Walk: enter/leave with multiplier ──

#[test]
fn walk_with_multiplier() {
    let ast = parse_definition_syntax("<length>+").unwrap();
    let mut types = Vec::new();

    walk_definition_syntax_enter(&ast, &mut |node| {
        types.push(node.node_type().to_string());
    });

    // Group > Multiplier > Type
    assert!(types.contains(&"Multiplier".to_string()));
    assert!(types.contains(&"Type".to_string()));
}

// ── Parse tests from definition-syntax-parse.js ──

#[test]
fn parse_simple_keyword() {
    let node = parse_definition_syntax("foo").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo");
}

#[test]
fn parse_type_reference() {
    let node = parse_definition_syntax("<length>").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<length>");
}

#[test]
fn parse_property_reference() {
    let node = parse_definition_syntax("<'color'>").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<'color'>");
}

#[test]
fn parse_bar_combinator() {
    let node = parse_definition_syntax("foo | bar").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo | bar");
}

#[test]
fn parse_double_bar_combinator() {
    let node = parse_definition_syntax("foo || bar").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo || bar");
}

#[test]
fn parse_double_ampersand_combinator() {
    let node = parse_definition_syntax("foo && bar").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo && bar");
}

#[test]
fn parse_space_combinator() {
    let node = parse_definition_syntax("foo bar").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo bar");
}

#[test]
fn parse_multiplier_star() {
    let node = parse_definition_syntax("foo*").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo*");
}

#[test]
fn parse_multiplier_plus() {
    let node = parse_definition_syntax("foo+").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo+");
}

#[test]
fn parse_multiplier_question() {
    let node = parse_definition_syntax("foo?").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo?");
}

#[test]
fn parse_multiplier_hash() {
    let node = parse_definition_syntax("foo#").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo#");
}

#[test]
fn parse_multiplier_range() {
    let node = parse_definition_syntax("foo{2,4}").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "foo{2,4}");
}

#[test]
fn parse_explicit_group() {
    let node = parse_definition_syntax("[ foo | bar ]").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "[ foo | bar ]");
}

#[test]
fn parse_explicit_group_with_bang() {
    let node = parse_definition_syntax("[ foo bar ]!").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "[ foo bar ]!");
}

#[test]
fn parse_at_keyword() {
    let node = parse_definition_syntax("@media").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "@media");
}

#[test]
fn parse_comma() {
    let node = parse_definition_syntax("foo , bar").unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert!(output.contains("foo"));
    assert!(output.contains("bar"));
    assert!(output.contains(","));
}

#[test]
fn parse_complex_definition() {
    // A real-world-ish definition
    let syntax = "<length> | <percentage> | auto";
    let node = parse_definition_syntax(syntax).unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<length> | <percentage> | auto");
}

#[test]
fn parse_mixed_combinators() {
    // Space has higher precedence than |
    let syntax = "a b | c";
    let node = parse_definition_syntax(syntax).unwrap();
    let output = generate_definition_syntax(&node, &DefSyntaxGenOptions::default());
    assert_eq!(output, "a b | c");
}

#[test]
fn parse_error_trailing_combinator() {
    let result = parse_definition_syntax("a |");
    assert!(result.is_err());
}

#[test]
fn parse_error_leading_combinator() {
    let result = parse_definition_syntax("| a");
    assert!(result.is_err());
}

// ── definition-syntax-generate.js: forceBraces ──

#[test]
fn generate_with_force_braces() {
    // definition-syntax-generate.js: "using forceBraces"
    let ast = parse_definition_syntax("a b | c || d && e f").unwrap();
    let opts = DefSyntaxGenOptions { force_braces: true, compact: false };
    let result = generate_definition_syntax(&ast, &opts);
    assert_eq!(result, "[ [ a b ] | [ c || [ d && [ e f ] ] ] ]");
}

// ── definition-syntax-generate.js: round-trip ──

#[test]
fn generate_round_trip_multipliers() {
    // Round-trip: parse → generate should preserve multipliers
    for input in &["a?", "a*", "a+", "a#", "a{1,3}"] {
        let ast = parse_definition_syntax(input).unwrap();
        let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
        assert_eq!(&output, input, "Round-trip failed for {input}");
    }
}

#[test]
fn generate_round_trip_combinators() {
    for input in &["a | b", "a || b", "a && b", "a b"] {
        let ast = parse_definition_syntax(input).unwrap();
        let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
        assert_eq!(&output, input, "Round-trip failed for {input}");
    }
}

#[test]
fn generate_round_trip_types() {
    for input in &["<length>", "<'color'>", "<foo()>", "<number [0,100]>"] {
        let ast = parse_definition_syntax(input).unwrap();
        let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
        assert_eq!(&output, input, "Round-trip failed for {input}");
    }
}

// ── definition-syntax-parse.js: bad syntax errors ──

#[test]
fn parse_error_expected_quote() {
    let result = parse_definition_syntax("<'>");
    assert!(result.is_err());
}

#[test]
fn parse_error_empty_angle_brackets() {
    let result = parse_definition_syntax("<>");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_hash() {
    let result = parse_definition_syntax("#");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_question() {
    let result = parse_definition_syntax("?");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_plus() {
    let result = parse_definition_syntax("+");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_star() {
    let result = parse_definition_syntax("*");
    assert!(result.is_err());
}

#[test]
fn parse_error_unexpected_exclamation() {
    let result = parse_definition_syntax("!");
    assert!(result.is_err());
}

#[test]
fn parse_error_unmatched_bracket() {
    let result = parse_definition_syntax("[]]");
    assert!(result.is_err());
}

#[test]
fn parse_error_unclosed_angle() {
    let result = parse_definition_syntax("<a");
    assert!(result.is_err());
}

#[test]
fn parse_error_unclosed_bracket() {
    let result = parse_definition_syntax("[a");
    assert!(result.is_err());
}
