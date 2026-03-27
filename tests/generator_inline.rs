//! Inline tests ported from `tests/fixtures/inline/generate.json`.
//!
//! Each test maps to a JS `it()` block from the csstree generate test suite.

use csstree::generator::{generate, GenerateOptions};
use csstree::parser::{parse, ParseOptions};
use csstree::parser::options::ParseFlags;

/// Helper: parse CSS then generate back.
fn round_trip(css: &str) -> String {
    let ast = parse(css, ParseOptions::default());
    generate(&ast, &GenerateOptions::default())
}

// ── generate ──

#[test]
fn generate_simple_css() {
    // line 56: ".a {\n  color: red;\n}\n"
    let result = round_trip(".a {\n  color: red;\n}\n");
    assert_eq!(result, ".a{color:red}");
}

// ── generate > sourceMap ──

#[test]
fn source_map_simple() {
    // line 66: ".a {\n  color: red;\n}\n" — generate should produce minified CSS
    let result = round_trip(".a {\n  color: red;\n}\n");
    assert_eq!(result, ".a{color:red}");
}

#[test]
fn source_map_complex_css() {
    // line 77: complex multi-rule + @media CSS
    let css = ".a { color: #ff0000; } .b { display: block; float: left; } @media foo { .c { color: red } }";
    let result = round_trip(css);
    assert_eq!(
        result,
        ".a{color:#ff0000}.b{display:block;float:left}@media foo{.c{color:red}}"
    );
}

// ── generate > auto-emit whitespace ──

#[test]
fn auto_whitespace_insert() {
    // line 121: "span#foo { border: 1%var(--a)#ff0000; }"
    // The generator should auto-insert whitespace where necessary
    let result = round_trip("span#foo { border: 1%var(--a)#ff0000; }");
    assert_eq!(result, "span#foo{border:1% var(--a) #ff0000}");
}

#[test]
fn auto_whitespace_tokenize_no_insert() {
    // line 127: with parseRulePrelude:false + parseValue:false, values are Raw tokens
    // Raw values go through tokenize() which should NOT auto-insert whitespace
    let opts = ParseOptions {
        flags: ParseFlags {
            parse_rule_prelude: false,
            parse_value: false,
            ..ParseFlags::default()
        },
        ..ParseOptions::default()
    };
    let ast = parse("span#foo { border: 1%var(--a)#ff0000; }", opts);
    let result = generate(&ast, &GenerateOptions::default());
    assert_eq!(result, "span#foo{border:1%var(--a)#ff0000}");
}

// ── generate > parse-options round-trip tests ──

#[test]
fn round_trip_parse_value_false() {
    // Verify parseValue:false produces same generate output
    let css = ".a { color: red; margin: 10px 20px }";
    let opts = ParseOptions {
        flags: ParseFlags {
            parse_value: false,
            ..ParseFlags::default()
        },
        ..ParseOptions::default()
    };
    let ast = parse(css, opts);
    let result = generate(&ast, &GenerateOptions::default());
    assert_eq!(result, ".a{color:red;margin:10px 20px}");
}

#[test]
fn round_trip_parse_rule_prelude_false() {
    let css = ".a .b, .c { color: red }";
    let opts = ParseOptions {
        flags: ParseFlags {
            parse_rule_prelude: false,
            ..ParseFlags::default()
        },
        ..ParseOptions::default()
    };
    let ast = parse(css, opts);
    let result = generate(&ast, &GenerateOptions::default());
    assert_eq!(result, ".a .b, .c{color:red}");
}

#[test]
fn round_trip_parse_atrule_prelude_false() {
    let css = "@media screen and (min-width: 768px) { .a { color: red } }";
    let opts = ParseOptions {
        flags: ParseFlags {
            parse_atrule_prelude: false,
            ..ParseFlags::default()
        },
        ..ParseOptions::default()
    };
    let ast = parse(css, opts);
    let result = generate(&ast, &GenerateOptions::default());
    assert_eq!(result, "@media screen and (min-width: 768px){.a{color:red}}");
}
