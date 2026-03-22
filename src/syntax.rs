//! CSS Syntax — the top-level API that ties all modules together.
//!
//! Provides a unified interface for parsing, generating, walking, and
//! validating CSS. This is the Rust equivalent of `createSyntax()` from
//! `external/csstree/lib/syntax/create.js`.

use crate::ast::Node;
use crate::generator::{generate, GenerateOptions};
use crate::lexer::{Lexer, LexerConfig, MatchResult};
use crate::parser::{parse, ParseOptions};
use crate::walker::{self, WalkAction, WalkContext, WalkOptions};

/// Top-level CSS syntax configuration and API.
///
/// Ties together the parser, generator, walker, and lexer into a single
/// coherent API. This is the main entry point for the csstree-rs library.
#[derive(Debug)]
pub struct CssSyntax {
    /// The CSS lexer for value validation.
    pub lexer: Lexer,
}

impl CssSyntax {
    /// Create a new CSS syntax instance with default configuration.
    pub fn new() -> Self {
        Self {
            lexer: Lexer::new(LexerConfig {
                generic: true,
                ..LexerConfig::default()
            }),
        }
    }

    /// Create a new CSS syntax instance with custom lexer config.
    pub fn with_config(lexer_config: LexerConfig) -> Self {
        Self {
            lexer: Lexer::new(lexer_config),
        }
    }

    /// Parse a CSS string into an AST.
    pub fn parse(&self, css: &str) -> Node {
        parse(css, ParseOptions::default())
    }

    /// Parse with custom options.
    pub fn parse_with_options(&self, css: &str, options: ParseOptions) -> Node {
        parse(css, options)
    }

    /// Generate CSS from an AST node.
    pub fn generate(&self, node: &Node) -> String {
        generate(node, &GenerateOptions::default())
    }

    /// Generate CSS with custom options.
    pub fn generate_with_options(&self, node: &Node, options: &GenerateOptions) -> String {
        generate(node, options)
    }

    /// Walk an AST node depth-first.
    pub fn walk<'a, F>(&self, root: &'a Node, enter: F)
    where
        F: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
    {
        walker::walk(root, enter);
    }

    /// Walk with full options.
    pub fn walk_full<'a, E, L>(
        &self,
        root: &'a Node,
        options: &WalkOptions,
        enter: E,
        leave: L,
    )
    where
        E: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
        L: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
    {
        walker::walk_full(root, options, enter, leave);
    }

    /// Find the first node matching a predicate.
    pub fn find<'a, F>(&self, root: &'a Node, predicate: F) -> Option<&'a Node>
    where
        F: FnMut(&'a Node, &WalkContext<'a>) -> bool,
    {
        walker::find(root, predicate)
    }

    /// Find all nodes matching a predicate.
    pub fn find_all<'a, F>(&self, root: &'a Node, predicate: F) -> Vec<&'a Node>
    where
        F: FnMut(&'a Node, &WalkContext<'a>) -> bool,
    {
        walker::find_all(root, predicate)
    }

    /// Validate a property value.
    pub fn match_property(&mut self, name: &str, value: &str) -> MatchResult {
        self.lexer.match_property(name, value)
    }

    /// Fork this syntax with extensions.
    #[must_use]
    pub fn fork(&self, mut config: LexerConfig) -> Self {
        // Merge properties
        for name in self.lexer.property_names() {
            if !config.properties.contains_key(name) {
                // Copy from parent if not overridden
                // (In a full impl, we'd clone the parent's properties)
            }
        }
        config.generic = true;
        Self::with_config(config)
    }
}

impl Default for CssSyntax {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn syntax_parse_and_generate() {
        let syntax = CssSyntax::new();
        let ast = syntax.parse(".a { color: red }");
        let css = syntax.generate(&ast);
        assert_eq!(css, ".a{color:red}");
    }

    #[test]
    fn syntax_walk() {
        let syntax = CssSyntax::new();
        let ast = syntax.parse(".a { color: red }");
        let mut types = Vec::new();
        syntax.walk(&ast, |node, _ctx| {
            types.push(node.node_type().to_string());
            WalkAction::Continue
        });
        assert!(types.contains(&"StyleSheet".to_string()));
        assert!(types.contains(&"Rule".to_string()));
    }

    #[test]
    fn syntax_find() {
        let syntax = CssSyntax::new();
        let ast = syntax.parse(".a { color: red }");
        let found = syntax.find(&ast, |node, _ctx| node.node_type() == "ClassSelector");
        assert!(found.is_some());
    }

    #[test]
    fn syntax_find_all() {
        let syntax = CssSyntax::new();
        let ast = syntax.parse(".a { color: red }");
        let found = syntax.find_all(&ast, |node, _ctx| node.node_type() == "Declaration");
        assert_eq!(found.len(), 1);
    }

    #[test]
    fn syntax_default() {
        let syntax = CssSyntax::default();
        let ast = syntax.parse("a {}");
        assert_eq!(ast.node_type(), "StyleSheet");
    }

    #[test]
    fn syntax_with_lexer_config() {
        let mut config = LexerConfig::default();
        config.generic = true;
        config.properties.insert("display".into(), "block | none | flex".into());

        let mut syntax = CssSyntax::with_config(config);
        let result = syntax.match_property("display", "block");
        assert!(result.matched.is_some());
    }

    #[test]
    fn syntax_round_trip() {
        let syntax = CssSyntax::new();
        let css = ".a { color: red; display: block }";
        let ast = syntax.parse(css);
        let output = syntax.generate(&ast);
        // Round-trip should produce minified CSS
        assert_eq!(output, ".a{color:red;display:block}");
    }
}
