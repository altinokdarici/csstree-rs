//! Additional tests from definition-syntax-generate.js, definition-syntax-match.js,
//! and definition-syntax-walk.js that weren't covered by existing test files.

use csstree::definition_syntax::{
    parse_definition_syntax, generate_definition_syntax,
    walk_definition_syntax, walk_definition_syntax_enter,
    DefSyntaxGenOptions,
};
use csstree::definition_syntax::types::DefinitionSyntaxNode;
use std::cell::RefCell;

// ── definition-syntax-generate.js: round-trip lexer dictionary ──

#[test]
fn generate_round_trip_property_syntax() {
    // Round-trip common property syntaxes
    let syntaxes = [
        "none | <length>",
        "<color>",
        "normal | bold | lighter | bolder | <integer>",
        "block | inline | none | flex | grid",
        "<length> | <percentage> | auto",
    ];
    for syntax in &syntaxes {
        let ast = parse_definition_syntax(syntax).unwrap();
        let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
        assert_eq!(&output, syntax, "Round-trip failed for {syntax}");
    }
}

#[test]
fn generate_round_trip_type_syntax() {
    let syntaxes = [
        "<integer>",
        "<number>",
        "<length-percentage>",
    ];
    for syntax in &syntaxes {
        let ast = parse_definition_syntax(syntax).unwrap();
        let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
        assert_eq!(&output, syntax);
    }
}

// ── definition-syntax-generate.js: compact mode ──

#[test]
fn generate_compact_mode() {
    let ast = parse_definition_syntax("a | b | c").unwrap();
    let opts = DefSyntaxGenOptions { force_braces: false, compact: true };
    let output = generate_definition_syntax(&ast, &opts);
    assert_eq!(output, "a|b|c");
}

#[test]
fn generate_compact_with_group() {
    let ast = parse_definition_syntax("[ a | b ] c").unwrap();
    let opts = DefSyntaxGenOptions { force_braces: false, compact: true };
    let output = generate_definition_syntax(&ast, &opts);
    // Space combinator preserves space even in compact mode
    assert_eq!(output, "[a|b] c");
}

// ── definition-syntax-match.js: basic matching ──

#[test]
fn match_create_default_syntax() {
    // Verify parse_definition_syntax can parse common patterns
    let patterns = [
        "<number>",
        "<integer>",
        "<length>",
        "none | auto",
        "<color>",
        "normal | bold",
    ];
    for pat in &patterns {
        assert!(parse_definition_syntax(pat).is_ok(), "Failed to parse: {pat}");
    }
}

#[test]
fn match_broken_type_reference() {
    // Should not panic on invalid type references
    let result = parse_definition_syntax("<>");
    assert!(result.is_err());
}

#[test]
fn match_broken_property_reference() {
    // Empty property reference
    let result = parse_definition_syntax("<''>");
    assert!(result.is_err());
}

// ── definition-syntax-walk.js: walker coverage ──

#[test]
fn walk_visits_all_nodes() {
    let ast = parse_definition_syntax("a | b c").unwrap();
    let mut count = 0;
    walk_definition_syntax_enter(&ast, &mut |_node: &DefinitionSyntaxNode| {
        count += 1;
    });
    assert!(count >= 3, "Should visit at least 3 nodes, got {count}");
}

#[test]
fn walk_group_with_multiplier() {
    let ast = parse_definition_syntax("<number>+").unwrap();
    let mut count = 0;
    walk_definition_syntax_enter(&ast, &mut |_node: &DefinitionSyntaxNode| {
        count += 1;
    });
    assert!(count >= 2, "Should visit at least Group and Type, got {count}");
}

#[test]
fn walk_enter_leave_balanced() {
    let ast = parse_definition_syntax("a && b || c").unwrap();
    let mut enter_count = 0;
    let mut leave_count = 0;
    walk_definition_syntax(&ast, &mut |_node: &DefinitionSyntaxNode| {
        enter_count += 1;
    }, &mut |_node: &DefinitionSyntaxNode| {
        leave_count += 1;
    });
    assert_eq!(enter_count, leave_count, "Enter and leave counts should match");
}

// ── definition-syntax-parse.js: combinator precedence ──

#[test]
fn parse_combinator_precedence_space_vs_bar() {
    // Space has higher precedence than |
    let ast = parse_definition_syntax("a b | c d").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "a b | c d");
}

#[test]
fn parse_combinator_precedence_double_ampersand_vs_bar() {
    let ast = parse_definition_syntax("a && b | c && d").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "a && b | c && d");
}

#[test]
fn parse_combinator_precedence_double_bar_vs_bar() {
    let ast = parse_definition_syntax("a || b | c || d").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "a || b | c || d");
}

#[test]
fn parse_explicit_group_preserves_brackets() {
    let ast = parse_definition_syntax("[ a | b ] c").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "[ a | b ] c");
}

// ── definition-syntax-parse.js: number in braces ──

#[test]
fn parse_error_empty_braces() {
    let result = parse_definition_syntax("<x>{}");
    assert!(result.is_err());
}

#[test]
fn parse_error_braces_comma_only() {
    let result = parse_definition_syntax("<x>{,2}");
    assert!(result.is_err());
}

// ── Multiplier edge cases ──

#[test]
fn parse_hash_multiplier() {
    let ast = parse_definition_syntax("<number>#").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<number>#");
}

#[test]
fn parse_question_multiplier() {
    let ast = parse_definition_syntax("<number>?").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<number>?");
}

#[test]
fn parse_range_multiplier() {
    let ast = parse_definition_syntax("<number>{2,4}").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<number>{2,4}");
}

#[test]
fn parse_exact_multiplier() {
    let ast = parse_definition_syntax("<number>{3}").unwrap();
    let output = generate_definition_syntax(&ast, &DefSyntaxGenOptions::default());
    assert_eq!(output, "<number>{3}");
}
