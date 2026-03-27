//! Inline tests ported from `external/csstree/lib/__tests/parse.js`.
//!
//! Covers: onComment, onToken equivalents, positions, browser hacks,
//! and formattedMessage for long lines.

use csstree::ast::Node;
use csstree::generator::{generate, GenerateOptions};
use csstree::parser::{parse, ParseOptions, ParseContext};
use csstree::tokenizer::tokenize;
use csstree::tokenizer::types::TokenType;
use csstree::walker::{walk, WalkAction};

// ── onComment with no locations (line 342) ──
// In Rust, we don't have an onComment callback in the same way.
// Test that comments don't affect the generated output.

#[test]
fn comments_do_not_affect_output() {
    let css = "/*123*/.foo[a=/* 234 */] {\n  color: red; /* 345*/\n  background: url(foo);\n} /*567*/";
    let ast = parse(css, ParseOptions::default());
    let output = generate(&ast, &GenerateOptions::default());
    // Generated output should contain the meaningful CSS, not the comments
    assert!(output.contains("color:red"), "output should contain color:red, got: {output}");
}

// ── onComment with locations (line 358) ──
// Test that parsing with positions=true does not crash with comments

#[test]
fn comments_with_positions_enabled() {
    let css = "/*123*/.foo[a=b] {\n  color: red; /* 345*/\n}";
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    opts.filename = Some("test.css".to_string());
    let ast = parse(css, opts);
    assert_eq!(ast.node_type(), "StyleSheet");

    // Verify positions are present
    match &ast {
        Node::StyleSheet(ss) => {
            assert!(ss.loc.is_some(), "StyleSheet should have location with positions=true");
            let loc = ss.loc.as_ref().unwrap();
            assert_eq!(loc.start.line, 1);
            assert_eq!(loc.start.column, 1);
        }
        _ => panic!("expected StyleSheet"),
    }
}

// ── onToken as function (line 417) ──
// Use tokenizer::tokenize() directly to collect tokens

#[test]
fn tokenize_collects_tokens() {
    let source = "/*123*/.foo[a=/* 234 */] {\n  color: red;} /*567*/";
    let mut tokens = Vec::new();

    tokenize(source, |token_type, start, end| {
        tokens.push((token_type, start, end));
    });

    // Should have multiple tokens
    assert!(!tokens.is_empty(), "tokenize should produce tokens");

    // First token should be a Comment
    assert_eq!(tokens[0].0, TokenType::Comment, "First token should be Comment");
    assert_eq!(tokens[0].1, 0, "First token start should be 0");
    assert_eq!(tokens[0].2, 7, "First token end should be 7");
}

// ── onToken as function with API (line 431) ──
// Verify token types and positions match expected values

#[test]
fn tokenize_token_types_and_positions() {
    let source = "/*123*/.foo[a=/* 234 */] {\n  color: red;} /*567*/";
    let mut tokens = Vec::new();

    tokenize(source, |token_type, start, end| {
        tokens.push((token_type, start, end));
    });

    // Verify specific tokens from the expected list in parse.js
    let expected: Vec<(TokenType, usize, usize)> = vec![
        (TokenType::Comment, 0, 7),         // /*123*/
        (TokenType::Delim, 7, 8),           // .
        (TokenType::Ident, 8, 11),          // foo
        (TokenType::LeftSquareBracket, 11, 12), // [
        (TokenType::Ident, 12, 13),         // a
        (TokenType::Delim, 13, 14),         // =
        (TokenType::Comment, 14, 23),       // /* 234 */
        (TokenType::RightSquareBracket, 23, 24), // ]
        (TokenType::WhiteSpace, 24, 25),    // space
        (TokenType::LeftCurlyBracket, 25, 26), // {
        (TokenType::WhiteSpace, 26, 29),    // \n  (whitespace)
        (TokenType::Ident, 29, 34),         // color
        (TokenType::Colon, 34, 35),         // :
        (TokenType::WhiteSpace, 35, 36),    // space
        (TokenType::Ident, 36, 39),         // red
        (TokenType::Semicolon, 39, 40),     // ;
        (TokenType::RightCurlyBracket, 40, 41), // }
        (TokenType::WhiteSpace, 41, 42),    // space
        (TokenType::Comment, 42, 49),       // /*567*  (unterminated — Rust tokenizer reads to source.len()+1)
    ];

    assert_eq!(
        tokens.len(),
        expected.len(),
        "Token count mismatch: got {} expected {}",
        tokens.len(),
        expected.len()
    );

    for (i, (exp_type, exp_start, exp_end)) in expected.iter().enumerate() {
        assert_eq!(
            tokens[i].0, *exp_type,
            "Token {i} type mismatch: got {:?} expected {:?}",
            tokens[i].0, exp_type
        );
        assert_eq!(
            tokens[i].1, *exp_start,
            "Token {i} start mismatch: got {} expected {exp_start}",
            tokens[i].1
        );
        assert_eq!(
            tokens[i].2, *exp_end,
            "Token {i} end mismatch: got {} expected {exp_end}",
            tokens[i].2
        );
    }
}

// ── onToken as array (line 479) ──
// Collect all tokens into a vector and verify basic properties

#[test]
fn tokenize_collect_all_tokens() {
    let source = ".foo { color: red; }";
    let mut tokens: Vec<(TokenType, usize, usize)> = Vec::new();

    tokenize(source, |token_type, start, end| {
        tokens.push((token_type, start, end));
    });

    // Verify we have reasonable tokens
    assert!(tokens.len() >= 8, "Should have at least 8 tokens, got {}", tokens.len());

    // First token is Delim (the dot)
    assert_eq!(tokens[0].0, TokenType::Delim);
    // Second token is Ident (foo)
    assert_eq!(tokens[1].0, TokenType::Ident);
    // Tokens should cover the entire source
    assert_eq!(tokens[0].1, 0, "First token should start at 0");
    assert_eq!(tokens.last().unwrap().2, source.len(), "Last token should end at source length");
}

// ── positions: start with line 1 column 1 (line 491) ──

#[test]
fn positions_start_with_line_1_column_1() {
    let source = ".foo.bar {\n  property: value 123 123.4 .123 123px 99% #fff url( a ) / var( --a ), \"test\" 'test';\n}";
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    let ast = parse(source, opts);

    let mut positions = Vec::new();
    walk(&ast, |node, _ctx| {
        if let Some(loc) = node_loc(node) {
            positions.push((loc.start.offset, loc.start.line, loc.start.column, node.node_type().to_string()));
        }
        WalkAction::Continue
    });

    // The first position should be the StyleSheet at offset 0, line 1, column 1
    assert!(!positions.is_empty(), "Should have positions");
    assert_eq!(positions[0], (0, 1, 1, "StyleSheet".to_string()));

    // Rule should also start at 0,1,1
    assert!(positions.iter().any(|p| p.3 == "Rule" && p.0 == 0 && p.1 == 1 && p.2 == 1));

    // SelectorList should start at 0,1,1
    assert!(positions.iter().any(|p| p.3 == "SelectorList" && p.0 == 0 && p.1 == 1 && p.2 == 1));

    // First ClassSelector at offset 0
    let class_selectors: Vec<_> = positions.iter().filter(|p| p.3 == "ClassSelector").collect();
    assert!(class_selectors.len() >= 2, "Should have at least 2 ClassSelectors");
    assert_eq!(class_selectors[0].0, 0, "First ClassSelector at offset 0");
    assert_eq!(class_selectors[1].0, 4, "Second ClassSelector at offset 4");

    // Block should start at offset 9 (after ".foo.bar ")
    assert!(positions.iter().any(|p| p.3 == "Block" && p.0 == 9));

    // Declaration should start on line 2
    let decl = positions.iter().find(|p| p.3 == "Declaration").unwrap();
    assert_eq!(decl.1, 2, "Declaration should be on line 2");
}

// ── positions: start with specified offset/line/column (line 535) ──

#[test]
fn positions_start_with_custom_offset_line_column() {
    let source = ".foo.bar {\n  property: value 123 123.4 .123 123px 99% #fff url( a ) / var( --a ), \"test\" 'test';\n}";
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    opts.offset = 100;
    opts.line = 3;
    opts.column = 5;
    let ast = parse(source, opts);

    let mut positions = Vec::new();
    walk(&ast, |node, _ctx| {
        if let Some(loc) = node_loc(node) {
            positions.push((loc.start.offset, loc.start.line, loc.start.column, node.node_type().to_string()));
        }
        WalkAction::Continue
    });

    assert!(!positions.is_empty(), "Should have positions");

    // StyleSheet should start at the custom offset/line/column
    assert_eq!(positions[0], (100, 3, 5, "StyleSheet".to_string()));

    // Rule should also start at 100, 3, 5
    assert!(positions.iter().any(|p| p.3 == "Rule" && p.0 == 100 && p.1 == 3 && p.2 == 5));

    // Second ClassSelector should be at offset 104 (100 + 4)
    let class_selectors: Vec<_> = positions.iter().filter(|p| p.3 == "ClassSelector").collect();
    assert!(class_selectors.len() >= 2);
    assert_eq!(class_selectors[1].0, 104, "Second ClassSelector at offset 104");
    assert_eq!(class_selectors[1].1, 3, "Second ClassSelector on line 3");

    // Declaration should be on line 4 (3 + 1 newline)
    let decl = positions.iter().find(|p| p.3 == "Declaration").unwrap();
    assert_eq!(decl.1, 4, "Declaration should be on line 4 (custom start line 3 + 1)");
}

// ── browser hack: other hack prefixes (line 593) ──

#[test]
fn browser_hack_dollar_prefix() {
    let ast = parse(".test { $color: value; }", ParseOptions::default());
    let decl = first_block_child(&ast);
    assert_eq!(decl.node_type(), "Declaration");
    if let Node::Declaration(d) = decl { assert_eq!(d.property, "$color"); }
}

#[test]
fn browser_hack_plus_prefix() {
    let ast = parse(".test { +width: value; }", ParseOptions::default());
    let decl = first_block_child(&ast);
    assert_eq!(decl.node_type(), "Declaration");
    if let Node::Declaration(d) = decl { assert_eq!(d.property, "+width"); }
}

#[test]
fn browser_hack_ampersand_prefix() {
    let ast = parse(".test { &margin: value; }", ParseOptions::default());
    let decl = first_block_child(&ast);
    assert_eq!(decl.node_type(), "Declaration");
    if let Node::Declaration(d) = decl { assert_eq!(d.property, "&margin"); }
}

#[test]
fn browser_hack_star_prefix() {
    let ast = parse(".test { *zoom: value; }", ParseOptions::default());
    let decl = first_block_child(&ast);
    assert_eq!(decl.node_type(), "Declaration");
    if let Node::Declaration(d) = decl { assert_eq!(d.property, "*zoom"); }
}

// ── formattedMessage for source with long lines (line 285) ──
// Verify that parsing CSS with very long lines doesn't crash

#[test]
fn parse_long_lines_no_crash() {
    let long_comment = format!("/*{}*/", "1234567890".repeat(20));
    let source = format!(
        "{}\n{}.{}\nfoo\n{}bar",
        long_comment,
        " ".repeat(117),
        "",
        " ".repeat(120)
    );
    let ast = parse(&source, ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

#[test]
fn parse_long_lines_with_positions() {
    let long_comment = format!("/*{}*/", "1234567890".repeat(20));
    let source = format!(
        "{}\n{}.{}\nfoo\n{}bar",
        long_comment,
        " ".repeat(117),
        "",
        " ".repeat(120)
    );
    let mut opts = ParseOptions::default();
    opts.flags.positions = true;
    let ast = parse(&source, opts);
    assert_eq!(ast.node_type(), "StyleSheet");
    // Verify locations are present
    match &ast {
        Node::StyleSheet(ss) => {
            assert!(ss.loc.is_some(), "Should have location info for long-line source");
        }
        _ => panic!("expected StyleSheet"),
    }
}

// ── Additional parse context tests ──

#[test]
fn parse_declaration_context() {
    let ast = parse("property: value", ParseOptions {
        context: ParseContext::Declaration,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "Declaration");
    if let Node::Declaration(d) = &ast {
        assert_eq!(d.property, "property");
    }
}

#[test]
fn parse_value_context() {
    let ast = parse("red blue", ParseOptions {
        context: ParseContext::Value,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "Value");
}

#[test]
fn parse_selector_list_context() {
    let ast = parse(".a, .b", ParseOptions {
        context: ParseContext::SelectorList,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "SelectorList");
}

#[test]
fn parse_selector_context() {
    let ast = parse(".a.b", ParseOptions {
        context: ParseContext::Selector,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "Selector");
}

#[test]
fn parse_declaration_list_context() {
    let ast = parse("foo: a; bar: b", ParseOptions {
        context: ParseContext::DeclarationList,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "DeclarationList");
}

#[test]
fn parse_block_context() {
    let ast = parse("{ color: red; }", ParseOptions {
        context: ParseContext::Block,
        ..ParseOptions::default()
    });
    assert_eq!(ast.node_type(), "Block");
}

// ── Tokenizer: verify substring extraction ──

#[test]
fn tokenize_substring_values() {
    let source = ".foo { color: red; }";
    let mut values = Vec::new();

    tokenize(source, |_token_type, start, end| {
        values.push(source[start..end].to_string());
    });

    assert!(values.contains(&".".to_string()));
    assert!(values.contains(&"foo".to_string()));
    assert!(values.contains(&"color".to_string()));
    assert!(values.contains(&"red".to_string()));
}

// ── Helper functions ──

fn node_loc(node: &Node) -> Option<&csstree::ast::Loc> {
    match node {
        Node::StyleSheet(n) => n.loc.as_ref(),
        Node::Rule(n) => n.loc.as_ref(),
        Node::SelectorList(n) => n.loc.as_ref(),
        Node::Selector(n) => n.loc.as_ref(),
        Node::ClassSelector(n) => n.loc.as_ref(),
        Node::Block(n) => n.loc.as_ref(),
        Node::Declaration(n) => n.loc.as_ref(),
        Node::Value(n) => n.loc.as_ref(),
        Node::Identifier(n) => n.loc.as_ref(),
        Node::Number(n) => n.loc.as_ref(),
        Node::Dimension(n) => n.loc.as_ref(),
        Node::Percentage(n) => n.loc.as_ref(),
        Node::Hash(n) => n.loc.as_ref(),
        Node::Url(n) => n.loc.as_ref(),
        Node::Operator(n) => n.loc.as_ref(),
        Node::Function(n) => n.loc.as_ref(),
        Node::StringNode(n) => n.loc.as_ref(),
        Node::Atrule(n) => n.loc.as_ref(),
        Node::AtrulePrelude(n) => n.loc.as_ref(),
        Node::Combinator(n) => n.loc.as_ref(),
        Node::TypeSelector(n) => n.loc.as_ref(),
        _ => None,
    }
}

fn first_block_child(ast: &Node) -> &Node {
    match ast {
        Node::StyleSheet(ss) => {
            match &ss.children[0] {
                Node::Rule(r) => {
                    match r.block.as_ref() {
                        Node::Block(b) => &b.children[0],
                        _ => panic!("Expected Block"),
                    }
                }
                _ => panic!("Expected Rule"),
            }
        }
        _ => panic!("Expected StyleSheet"),
    }
}
