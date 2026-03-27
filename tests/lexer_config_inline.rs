//! Tests ported from `external/csstree/lib/__tests/lexer.js` and
//! `external/csstree/lib/__tests/lexer-match-result.js`.
//!
//! Tests lexer configuration: validate, dump/recovery, CSS-wide keywords,
//! units override, and match result trace API.

use csstree::lexer::{Lexer, LexerConfig, ValidationResult};
use std::collections::HashMap;

fn make_basic_lexer() -> Lexer {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("color".into(), "<hex-color> | <ident>".into());
    config.properties.insert("display".into(), "block | inline | none".into());
    config.properties.insert("background".into(), "<hex-color> | <ident> | none".into());
    Lexer::new(config)
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should not override generic types when used
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn generic_types_not_overridden() {
    // When generic=true, built-in generics take priority over custom type definitions
    let mut config = LexerConfig::default();
    config.generic = true;
    config.types.insert("length".into(), "foo".into());
    let lexer = Lexer::new(config);

    // "foo" should NOT match — generic <length> takes priority
    assert!(lexer.match_type("length", "foo").matched.is_none(),
        "custom type 'foo' should not override generic <length>");
    // "1px" SHOULD match — generic <length> handles it
    assert!(lexer.match_type("length", "1px").matched.is_some(),
        "1px should match generic <length>");
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should not use generic type names when generics are not used
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn custom_types_when_no_generics() {
    // When generic=false, custom type "length" IS the definition
    let mut config = LexerConfig::default();
    config.generic = false;
    config.types.insert("length".into(), "foo".into());
    let lexer = Lexer::new(config);

    assert!(lexer.match_type("length", "foo").matched.is_some());
    assert!(lexer.match_type("length", "1px").matched.is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: validate()
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn validate_valid_config_returns_none() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.types.insert("ref".into(), "<number>".into());
    config.types.insert("valid".into(), "<number> <ref>".into());
    config.properties.insert("valid".into(), "<ident> <'ref'>".into());
    config.properties.insert("ref".into(), "<valid>".into());
    let lexer = Lexer::new(config);

    // Valid part should have no errors for these entries
    // (the validate checks for missing refs)
    let result = lexer.validate();
    // With generic=true, <number> is built-in, so ref and valid should be fine
    assert!(result.is_none(), "Expected no validation errors, got {:?}", result);
}

#[test]
fn validate_invalid_config_returns_errors() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.types.insert("invalid".into(), "<foo>".into()); // <foo> doesn't exist
    let lexer = Lexer::new(config);

    let result = lexer.validate();
    assert!(result.is_some(), "Expected validation errors");
    let result = result.unwrap();
    assert!(!result.errors.is_empty());
    assert!(result.types.contains(&"invalid".to_string()));
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should allow override units
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn override_units() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.units.insert("length".into(), vec!["xx".into(), "yy".into()]);
    let lexer = Lexer::new(config);

    assert_eq!(lexer.units().get("length").unwrap(), &vec!["xx".to_string(), "yy".to_string()]);
    // With custom units, px should no longer be a valid length unit
    // (the generic matcher checks the units map)
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should not add new unit groups or discard existing
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn no_new_unit_groups() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.units.insert("foo".into(), vec!["xx".into(), "yy".into()]);
    let lexer = Lexer::new(config);

    // "foo" should NOT be added (not an existing group)
    assert!(!lexer.units().contains_key("foo"));
    // Existing groups should still be present
    assert!(lexer.units().contains_key("length"));
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should allow to override CSS wide keywords
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn override_css_wide_keywords() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.css_wide_keywords = Some(vec!["foo".into(), "bar".into()]);
    config.properties.insert("test".into(), "<number>".into());
    let lexer = Lexer::new(config);

    // Custom keywords should match
    assert!(lexer.match_property("test", "foo").matched.is_some());
    assert!(lexer.match_property("test", "bar").matched.is_some());
    // Default keywords should NOT match
    assert!(lexer.match_property("test", "inherit").matched.is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should allow append definitions > properties
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn append_properties() {
    let mut lexer = make_basic_lexer();
    lexer.add_property("new-prop", "foo | bar");
    assert!(lexer.match_property("new-prop", "foo").matched.is_some());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: should allow append definitions > types
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn append_types() {
    let mut lexer = make_basic_lexer();
    lexer.add_type("new-type", "foo | bar");
    assert!(lexer.match_type("new-type", "foo").matched.is_some());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: default syntax shouldn't be broken
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn default_syntax_valid() {
    let lexer = make_basic_lexer();
    assert!(lexer.validate().is_none(), "Default lexer should validate cleanly");
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: dump & recovery > custom syntax should not affect base syntax
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dump_custom_does_not_affect_base() {
    let base = make_basic_lexer();
    // Base lexer should not know about custom properties
    assert!(base.match_property("test", "1 2 3").matched.is_none());
    // But should still work for its own properties
    assert!(base.match_property("color", "red").matched.is_some());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: dump & recovery > custom syntax should be valid and correct
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dump_custom_syntax_valid() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("test".into(), "<number>+".into());
    let lexer = Lexer::new(config);

    assert!(lexer.validate().is_none(), "Custom lexer should validate cleanly");
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: dump & recovery > custom syntax should match own grammar only
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dump_custom_matches_own_grammar() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("test".into(), "<number>+".into());
    let lexer = Lexer::new(config);

    assert!(lexer.match_property("test", "1").matched.is_some());
    // Should NOT know about "color" property (not in this config)
    assert!(lexer.match_property("color", "red").matched.is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer.js: dump & recovery > recovery syntax from dump
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn dump_and_recovery() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.css_wide_keywords = Some(vec!["wide".into()]);
    config.properties.insert("test".into(), "<number>".into());
    let lexer = Lexer::new(config);

    // Dump the config
    let dumped = lexer.dump();

    // Create a new lexer from the dumped config
    let recovered = Lexer::new(dumped);

    // Recovered lexer should validate
    assert!(recovered.validate().is_none());
    // Recovered lexer should match the same values
    assert!(recovered.match_property("test", "42").matched.is_some());
    // Recovered lexer should use custom CSS-wide keywords
    assert!(recovered.match_property("test", "wide").matched.is_some());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer-match-result.js: getTrace()
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn match_result_get_trace() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("background".into(), "<hex-color> | <ident>".into());
    let lexer = Lexer::new(config);

    let result = lexer.match_property("background", "red");
    assert!(result.matched.is_some());

    // Get trace for the first matched token (index 0)
    let trace = result.get_trace(0);
    assert!(trace.is_some(), "Should have trace for matched token");
}

#[test]
fn match_result_get_trace_mismatch_returns_empty() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.properties.insert("margin".into(), "<length>".into());
    let lexer = Lexer::new(config);

    let result = lexer.match_property("margin", "red");
    assert!(result.matched.is_none());
    // Mismatch has no trace
    assert!(result.get_trace(0).is_none());
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer-match-result.js: isType()
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn match_result_is_type() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.types.insert("my-color".into(), "<hex-color> | <ident>".into());
    config.properties.insert("background".into(), "<my-color>".into());
    let lexer = Lexer::new(config);

    let result = lexer.match_property("background", "red");
    assert!(result.matched.is_some());
    assert!(result.is_type(0, "my-color"), "Should detect my-color type in trace");
    assert!(!result.is_property(0, "my-color"), "my-color is type, not property");
}

// ══════════════════════════════════════════════════════════════════════════════
// lexer-match-result.js: isProperty()
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn match_result_is_property() {
    let mut config = LexerConfig::default();
    config.generic = true;
    config.types.insert("my-color".into(), "<hex-color> | <ident>".into());
    config.properties.insert("bg-color".into(), "<my-color>".into());
    config.properties.insert("background".into(), "<'bg-color'>".into());
    let lexer = Lexer::new(config);

    let result = lexer.match_property("background", "red");
    assert!(result.matched.is_some());
    assert!(result.is_property(0, "bg-color"), "Should detect bg-color property in trace");
    assert!(result.is_type(0, "my-color"), "Should detect my-color type in trace");
}
