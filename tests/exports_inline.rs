//! Export tests — verifies that all public modules and key types are accessible.
//!
//! Ported from: `external/csstree/lib/__tests/exports.js`

// ── tokenizer module ──

#[test]
fn tokenizer_module_has_tokenize() {
    // Verify the tokenize function is accessible and works
    use csstree::tokenizer::types::TokenType;
    let mut tokens = Vec::new();
    csstree::tokenizer::tokenize("a", |tt, start, end| {
        tokens.push((tt, start, end));
    });
    // Should produce at least an Ident token and EOF
    assert!(!tokens.is_empty());
    assert_eq!(tokens[0].0, TokenType::Ident);
}

#[test]
fn tokenizer_module_has_token_type() {
    // Verify TokenType enum variants exist and have correct discriminants
    use csstree::tokenizer::types::TokenType;
    assert_eq!(TokenType::Eof as u8, 0);
    assert_eq!(TokenType::Ident as u8, 1);
    assert_eq!(TokenType::Function as u8, 2);
    assert_eq!(TokenType::Hash as u8, 4);
    assert_eq!(TokenType::String as u8, 5);
    assert_eq!(TokenType::Number as u8, 10);
    assert_eq!(TokenType::Dimension as u8, 12);
    assert_eq!(TokenType::Percentage as u8, 11);
    assert_eq!(TokenType::WhiteSpace as u8, 13);
    assert_eq!(TokenType::Comment as u8, 25);
    assert_eq!(TokenType::Url as u8, 7);
    assert_eq!(TokenType::Delim as u8, 9);
}

// ── parser module ──

#[test]
fn parser_module_has_parse() {
    use csstree::parser::{parse, ParseOptions};
    let ast = parse("a{}", ParseOptions::default());
    assert_eq!(ast.node_type(), "StyleSheet");
}

#[test]
fn parser_module_has_parse_options() {
    use csstree::parser::{ParseContext, ParseOptions};
    let opts = ParseOptions::default();
    assert_eq!(opts.context, ParseContext::StyleSheet);
    assert!(!opts.flags.positions);
    assert!(opts.flags.parse_value);
}

// ── generator module ──

#[test]
fn generator_module_has_generate() {
    use csstree::generator::{generate, GenerateOptions};
    use csstree::parser::{parse, ParseOptions};

    let ast = parse(".a { color: red }", ParseOptions::default());
    let css = generate(&ast, &GenerateOptions::default());
    assert_eq!(css, ".a{color:red}");
}

#[test]
fn generator_module_has_generate_options() {
    use csstree::generator::{GenerateMode, GenerateOptions};

    let default = GenerateOptions::default();
    assert_eq!(default.mode, GenerateMode::Safe);

    let spec = GenerateOptions {
        mode: GenerateMode::Spec,
        ..GenerateOptions::default()
    };
    assert_eq!(spec.mode, GenerateMode::Spec);
}

// ── walker module ──

#[test]
fn walker_module_has_walk() {
    use csstree::parser::{parse, ParseOptions};
    use csstree::walker::{walk, WalkAction};

    let ast = parse("a{}", ParseOptions::default());
    let mut types = Vec::new();
    walk(&ast, |node, _ctx| {
        types.push(node.node_type().to_string());
        WalkAction::Continue
    });
    assert!(types.contains(&"StyleSheet".to_string()));
}

#[test]
fn walker_module_has_find() {
    use csstree::parser::{parse, ParseOptions};
    use csstree::walker::find;

    let ast = parse(".a { color: red }", ParseOptions::default());
    let found = find(&ast, |node, _ctx| node.node_type() == "ClassSelector");
    assert!(found.is_some());
}

#[test]
fn walker_module_has_find_all() {
    use csstree::parser::{parse, ParseOptions};
    use csstree::walker::find_all;

    let ast = parse(".a { color: red; display: block }", ParseOptions::default());
    let found = find_all(&ast, |node, _ctx| node.node_type() == "Declaration");
    assert_eq!(found.len(), 2);
}

// ── remaining modules ──

#[test]
fn ast_module_has_node() {
    use csstree::ast::{Identifier, Node};

    let node = Node::Identifier(Identifier {
        loc: None,
        name: "red".into(),
    });
    assert_eq!(node.node_type(), "Identifier");
}

#[test]
fn css_syntax_and_remaining_modules_exist() {
    // CssSyntax
    let syntax = csstree::CssSyntax::new();
    let ast = syntax.parse(".a { color: red }");
    assert_eq!(ast.node_type(), "StyleSheet");

    // Lexer module
    let lexer = csstree::lexer::Lexer::new(csstree::lexer::LexerConfig::default());
    assert!(lexer.property_names().is_empty()); // no custom properties loaded

    // definition_syntax module
    let ds_result = csstree::definition_syntax::parse::parse("<length> | auto");
    assert!(ds_result.is_ok());

    // utils module
    let kw = csstree::utils::keyword_descriptor("foo");
    assert_eq!(kw.basename, "foo");
    let vp = csstree::utils::vendor_prefix("-webkit-foo");
    assert_eq!(vp, "-webkit-");
    assert!(csstree::utils::is_custom_property("--foo"));
    assert!(!csstree::utils::is_custom_property("color"));
}
