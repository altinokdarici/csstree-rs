//! Inline tests for CSS lexer at-rule checking and matching.
//!
//! Ported from:
//! - external/csstree/lib/__tests/lexer-check-atrule-name.js
//! - external/csstree/lib/__tests/lexer-check-atrule-prelude.js
//! - external/csstree/lib/__tests/lexer-check-atrule-descriptor.js
//! - external/csstree/lib/__tests/lexer-match-atrule-prelude.js
//! - external/csstree/lib/__tests/lexer-match-atrule-descriptor.js

use std::collections::HashMap;
use csstree::lexer::{AtruleConfig, Lexer, LexerConfig};

/// Build a lexer with default at-rule definitions matching csstree's defaults.
fn make_lexer() -> Lexer {
    let mut config = LexerConfig::default();
    config.generic = true;

    // At-rule: font-face (no prelude, has descriptors)
    config.atrules.insert("font-face".into(), AtruleConfig {
        prelude: None,
        descriptors: {
            let mut d = HashMap::new();
            d.insert("font-display".into(), "auto | block | swap | fallback | optional".into());
            d.insert("font-family".into(), "<custom-ident>".into());
            d.insert("font-weight".into(), "<number>".into());
            d.insert("font-style".into(), "normal | italic | oblique".into());
            d.insert("font-stretch".into(), "<percentage>".into());
            d.insert("src".into(), "<ident>".into());
            d.insert("unicode-range".into(), "<ident>".into());
            d.insert("ascent-override".into(), "normal | <percentage>".into());
            d.insert("descent-override".into(), "normal | <percentage>".into());
            d.insert("line-gap-override".into(), "normal | <percentage>".into());
            d.insert("size-adjust".into(), "<percentage>".into());
            d.insert("font-feature-settings".into(), "normal | <ident>".into());
            d.insert("font-variation-settings".into(), "normal | <ident>".into());
            d
        },
    });

    // At-rule: keyframes (has prelude, no descriptors)
    config.atrules.insert("keyframes".into(), AtruleConfig {
        prelude: Some("<custom-ident>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: media (has prelude, no descriptors)
    config.atrules.insert("media".into(), AtruleConfig {
        prelude: Some("<ident>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: page (has prelude that allows empty, has descriptors)
    config.atrules.insert("page".into(), AtruleConfig {
        prelude: Some("<ident>?".into()),
        descriptors: {
            let mut d = HashMap::new();
            d.insert("size".into(), "<length>".into());
            d.insert("marks".into(), "none | crop | cross".into());
            d.insert("bleed".into(), "<length>".into());
            d.insert("page-orientation".into(), "upright | rotate-left | rotate-right".into());
            d
        },
    });

    // At-rule: import (has prelude, no descriptors)
    config.atrules.insert("import".into(), AtruleConfig {
        prelude: Some("<string>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: supports (has prelude, no descriptors)
    config.atrules.insert("supports".into(), AtruleConfig {
        prelude: Some("<ident>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: charset (has prelude, no descriptors)
    config.atrules.insert("charset".into(), AtruleConfig {
        prelude: Some("<string>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: namespace (has prelude, no descriptors)
    config.atrules.insert("namespace".into(), AtruleConfig {
        prelude: Some("<ident>".into()),
        descriptors: HashMap::new(),
    });

    // At-rule: starting-style (no prelude, no descriptors but entry exists)
    config.atrules.insert("starting-style".into(), AtruleConfig {
        prelude: None,
        descriptors: HashMap::new(),
    });

    Lexer::new(config)
}

// ══════════════════════════════════════════════════════════════════════════════
// checkAtruleName tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn check_atrule_name_valid_media() {
    let lexer = make_lexer();
    assert!(lexer.check_atrule_name("media").is_ok());
}

#[test]
fn check_atrule_name_vendor_prefix_webkit_keyframes() {
    let lexer = make_lexer();
    // Vendor-prefixed atrule should resolve to basename
    assert!(lexer.check_atrule_name("-webkit-keyframes").is_ok());
}

#[test]
fn check_atrule_name_vendor_prefix_webkit_media() {
    let lexer = make_lexer();
    // Even -webkit-media resolves to 'media' (same as JS — FIXME in upstream)
    assert!(lexer.check_atrule_name("-webkit-media").is_ok());
}

#[test]
fn check_atrule_name_invalid_foo() {
    let lexer = make_lexer();
    let err = lexer.check_atrule_name("foo").unwrap_err();
    assert_eq!(err.message, "Unknown at-rule `@foo`");
}

// ══════════════════════════════════════════════════════════════════════════════
// checkAtrulePrelude tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn check_atrule_prelude_invalid_atrule() {
    let mut lexer = make_lexer();
    let err = lexer.check_atrule_prelude("foo", None).unwrap_err();
    assert_eq!(err.message, "Unknown at-rule `@foo`");
}

#[test]
fn check_atrule_prelude_font_face_with_prelude_should_fail() {
    let mut lexer = make_lexer();
    let err = lexer.check_atrule_prelude("font-face", Some("hi")).unwrap_err();
    assert_eq!(err.message, "At-rule `@font-face` should not contain a prelude");
}

#[test]
fn check_atrule_prelude_font_face_empty_string_ok() {
    let mut lexer = make_lexer();
    assert!(lexer.check_atrule_prelude("font-face", Some("")).is_ok());
}

#[test]
fn check_atrule_prelude_font_face_none_ok() {
    let mut lexer = make_lexer();
    assert!(lexer.check_atrule_prelude("font-face", None).is_ok());
}

#[test]
fn check_atrule_prelude_page_empty_ok() {
    let mut lexer = make_lexer();
    assert!(lexer.check_atrule_prelude("page", Some("")).is_ok());
}

#[test]
fn check_atrule_prelude_page_none_ok() {
    let mut lexer = make_lexer();
    assert!(lexer.check_atrule_prelude("page", None).is_ok());
}

#[test]
fn check_atrule_prelude_keyframes_empty_should_fail() {
    let mut lexer = make_lexer();
    let err = lexer.check_atrule_prelude("keyframes", Some("")).unwrap_err();
    assert_eq!(err.message, "At-rule `@keyframes` should contain a prelude");
}

#[test]
fn check_atrule_prelude_keyframes_none_should_fail() {
    let mut lexer = make_lexer();
    let err = lexer.check_atrule_prelude("keyframes", None).unwrap_err();
    assert_eq!(err.message, "At-rule `@keyframes` should contain a prelude");
}

#[test]
fn check_atrule_prelude_keyframes_with_value_ok() {
    let mut lexer = make_lexer();
    assert!(lexer.check_atrule_prelude("keyframes", Some("test")).is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// checkAtruleDescriptorName tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn check_atrule_descriptor_name_invalid_atrule() {
    let lexer = make_lexer();
    let err = lexer.check_atrule_descriptor_name("foo", None).unwrap_err();
    assert_eq!(err.message, "Unknown at-rule `@foo`");
}

#[test]
fn check_atrule_descriptor_name_no_descriptors() {
    let lexer = make_lexer();
    let err = lexer.check_atrule_descriptor_name("import", Some("test")).unwrap_err();
    assert_eq!(err.message, "At-rule `@import` has no known descriptors");
}

#[test]
fn check_atrule_descriptor_name_unknown_descriptor() {
    let lexer = make_lexer();
    let err = lexer.check_atrule_descriptor_name("font-face", Some("color")).unwrap_err();
    assert_eq!(err.message, "Unknown at-rule descriptor `color`");
}

#[test]
fn check_atrule_descriptor_name_valid() {
    let lexer = make_lexer();
    assert!(lexer.check_atrule_descriptor_name("font-face", Some("font-family")).is_ok());
}

// ══════════════════════════════════════════════════════════════════════════════
// matchAtruleDescriptor tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn match_atrule_descriptor_basic_match() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "swap");
    assert!(result.matched.is_some(), "Expected match for font-display: swap");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_descriptor_vendor_prefix_atrule() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("-prefix-font-face", "font-display", "swap");
    assert!(result.matched.is_some(), "Expected match with vendor-prefixed atrule");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_descriptor_vendor_prefix_descriptor() {
    let mut lexer = make_lexer();
    // Vendor-prefixed descriptor falls back to basename
    let result = lexer.match_atrule_descriptor("font-face", "-prefix-font-display", "swap");
    assert!(result.matched.is_some(), "Expected match with vendor-prefixed descriptor");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_descriptor_case_insensitive() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("FONT-FACE", "FONT-DISPLAY", "swap");
    assert!(result.matched.is_some(), "Expected case-insensitive match");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_descriptor_empty_value_mismatch() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "");
    assert!(result.matched.is_none(), "Empty value should not match");
}

#[test]
fn match_atrule_descriptor_no_descriptors_error() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("keyframes", "font-face", "swap");
    assert!(result.matched.is_none());
    assert_eq!(
        result.error.as_deref(),
        Some("At-rule `@keyframes` has no known descriptors")
    );
}

#[test]
fn match_atrule_descriptor_css_wide_keyword_initial() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "initial");
    assert!(result.matched.is_none(), "CSS-wide keywords should not match for descriptors");
}

#[test]
fn match_atrule_descriptor_css_wide_keyword_inherit() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "inherit");
    assert!(result.matched.is_none(), "CSS-wide keywords should not match for descriptors");
}

#[test]
fn match_atrule_descriptor_css_wide_keyword_unset() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "unset");
    assert!(result.matched.is_none(), "CSS-wide keywords should not match for descriptors");
}

#[test]
fn match_atrule_descriptor_css_wide_keyword_revert() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "revert");
    assert!(result.matched.is_none(), "CSS-wide keywords should not match for descriptors");
}

#[test]
fn match_atrule_descriptor_css_wide_keyword_revert_layer() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_descriptor("font-face", "font-display", "revert-layer");
    assert!(result.matched.is_none(), "CSS-wide keywords should not match for descriptors");
}

// ══════════════════════════════════════════════════════════════════════════════
// matchAtrulePrelude tests
// ══════════════════════════════════════════════════════════════════════════════

#[test]
fn match_atrule_prelude_basic_match() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("keyframes", Some("test"));
    assert!(result.matched.is_some(), "Expected match for keyframes prelude");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_prelude_vendor_prefix() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("-webkit-keyframes", Some("test"));
    assert!(result.matched.is_some(), "Expected match with vendor prefix");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_prelude_case_insensitive() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("KEYFRAMES", Some("test"));
    assert!(result.matched.is_some(), "Expected case-insensitive match");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_prelude_case_insensitive_with_vendor() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("-VENDOR-Keyframes", Some("test"));
    assert!(result.matched.is_some(), "Expected case-insensitive match with vendor");
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_prelude_empty_value_mismatch() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("keyframes", Some(""));
    assert!(result.matched.is_none());
    assert!(result.error.is_some());
}

#[test]
fn match_atrule_prelude_font_face_null_positive() {
    let mut lexer = make_lexer();
    // No prelude and at-rule has no prelude = positive with null matched
    let result = lexer.match_atrule_prelude("font-face", None);
    assert!(result.matched.is_none());
    assert!(result.error.is_none());
}

#[test]
fn match_atrule_prelude_font_face_with_value_error() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("font-face", Some("test"));
    assert!(result.matched.is_none());
    assert_eq!(
        result.error.as_deref(),
        Some("At-rule `@font-face` should not contain a prelude")
    );
}

#[test]
fn match_atrule_prelude_vendor_font_face_with_value_error() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("-prefix-font-face", Some("test"));
    assert!(result.matched.is_none());
    assert_eq!(
        result.error.as_deref(),
        Some("At-rule `@-prefix-font-face` should not contain a prelude")
    );
}

#[test]
fn match_atrule_prelude_page_empty_ok() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("page", Some(""));
    assert!(result.error.is_none());
    assert!(result.matched.is_some());
    // Empty match
    assert!(result.matched.unwrap().is_empty());
}

#[test]
fn match_atrule_prelude_page_none_ok() {
    let mut lexer = make_lexer();
    let result = lexer.match_atrule_prelude("page", None);
    assert!(result.error.is_none());
    assert!(result.matched.is_some());
    assert!(result.matched.unwrap().is_empty());
}
