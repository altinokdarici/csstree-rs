//! Inline tests for the CSS lexer matching capability.
//!
//! Tests ported from `tests/fixtures/inline/lexer*.json` and JS test files.

use csstree::lexer::{Lexer, LexerConfig};

fn make_lexer() -> Lexer {
    let mut config = LexerConfig::default();
    config.generic = true;

    // Basic property definitions
    config.properties.insert("display".into(), "block | inline | none | flex | grid | inline-block | inline-flex | table".into());
    config.properties.insert("color".into(), "<hex-color> | <ident>".into());
    config.properties.insert("margin".into(), "<length> | <percentage> | auto".into());
    config.properties.insert("font-weight".into(), "normal | bold | bolder | lighter | <integer>".into());
    config.properties.insert("opacity".into(), "<number>".into());
    config.properties.insert("border".into(), "<length> || <ident> || <hex-color>".into());

    Lexer::new(config)
}

// ── matchProperty: keyword values ──

#[test]
fn match_property_display_block() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "block").matched.is_some());
}

#[test]
fn match_property_display_none() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "none").matched.is_some());
}

#[test]
fn match_property_display_flex() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "flex").matched.is_some());
}

#[test]
fn match_property_display_invalid() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "banana").matched.is_none());
}

// ── matchProperty: CSS-wide keywords ──

#[test]
fn match_property_initial() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "initial").matched.is_some());
}

#[test]
fn match_property_inherit() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "inherit").matched.is_some());
}

#[test]
fn match_property_unset() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "unset").matched.is_some());
}

#[test]
fn match_property_revert() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "revert").matched.is_some());
}

#[test]
fn match_property_revert_layer() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "revert-layer").matched.is_some());
}

// ── matchProperty: case insensitivity ──

#[test]
fn match_property_case_insensitive() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("display", "BLOCK").matched.is_some());
    assert!(lexer.match_property("display", "Block").matched.is_some());
    assert!(lexer.match_property("display", "NONE").matched.is_some());
}

// ── matchProperty: unknown property ──

#[test]
fn match_property_unknown() {
    let mut lexer = make_lexer();
    let result = lexer.match_property("nonexistent", "value");
    assert!(result.matched.is_none());
    assert!(result.error.is_some());
}

// ── matchProperty: number values ──

#[test]
fn match_property_opacity_number() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("opacity", "0.5").matched.is_some());
    assert!(lexer.match_property("opacity", "1").matched.is_some());
    assert!(lexer.match_property("opacity", "0").matched.is_some());
}

#[test]
fn match_property_opacity_keyword_invalid() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("opacity", "auto").matched.is_none());
}

// ── matchProperty: integer values ──

#[test]
fn match_property_font_weight_keyword() {
    let mut lexer = make_lexer();
    assert!(lexer.match_property("font-weight", "bold").matched.is_some());
    assert!(lexer.match_property("font-weight", "normal").matched.is_some());
}

// ── checkPropertyName ──

#[test]
fn check_property_name_valid() {
    let lexer = make_lexer();
    assert!(lexer.check_property_name("display").is_ok());
    assert!(lexer.check_property_name("color").is_ok());
}

#[test]
fn check_property_name_invalid() {
    let lexer = make_lexer();
    assert!(lexer.check_property_name("nonexistent").is_err());
}

// ── Dynamic registration ──

#[test]
fn add_property_then_match() {
    let mut lexer = make_lexer();
    lexer.add_property("my-custom", "foo | bar | baz");
    assert!(lexer.match_property("my-custom", "foo").matched.is_some());
    assert!(lexer.match_property("my-custom", "baz").matched.is_some());
    assert!(lexer.match_property("my-custom", "qux").matched.is_none());
}

#[test]
fn add_type_then_match() {
    let mut lexer = make_lexer();
    lexer.add_type("my-type", "a | b | c");
    let result = lexer.match_type("my-type", "b");
    assert!(result.matched.is_some());
}
