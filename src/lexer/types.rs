//! Core types for the CSS lexer match graph and matching.

use std::collections::HashMap;

// ── Match Graph Node Types ──

/// A node in the match graph (automaton for CSS value validation).
#[derive(Debug, Clone)]
pub enum MatchNode {
    /// Terminal success state.
    Match,
    /// Terminal failure state.
    Mismatch,
    /// Success only if match consumed tokens (prevents empty matches in loops).
    DisallowEmpty,

    /// Conditional branch node.
    If {
        /// The condition to test.
        condition: Box<MatchNode>,
        /// Branch if condition matches.
        then_branch: Box<MatchNode>,
        /// Branch if condition fails.
        else_branch: Box<MatchNode>,
    },

    /// Optimized keyword dispatch table.
    Enum {
        /// Map of lowercase keyword → match node.
        map: HashMap<String, MatchNode>,
    },

    /// Bitmask-based permutation matcher (for `&&`/`||` with many terms).
    MatchOnce {
        /// Terms to match against.
        terms: Vec<MatchNode>,
        /// Whether all terms must match (true=`&&`, false=`||`).
        all: bool,
    },

    /// Built-in type validation function reference.
    Generic {
        /// Name of the generic type (e.g. `"length"`, `"color"`).
        name: String,
    },

    /// Reference to a type definition.
    Type {
        /// Type name to look up.
        name: String,
    },

    /// Reference to a property definition.
    Property {
        /// Property name to look up.
        name: String,
    },

    /// Literal keyword match (case-insensitive).
    Keyword {
        /// The keyword to match.
        name: String,
    },

    /// At-keyword match (e.g. `@media`).
    AtKeyword {
        /// The at-keyword name.
        name: String,
    },

    /// Function name match (e.g. `rgb(`).
    Function {
        /// The function name.
        name: String,
    },

    /// Exact token match (case-sensitive).
    Token {
        /// The token value to match.
        value: String,
    },

    /// Comma separator with context validation.
    Comma,

    /// Multi-token string match (case-insensitive).
    StringMatch {
        /// The string to match.
        value: String,
    },

    /// Wrapper for an entire match graph.
    MatchGraph {
        /// The root node of the graph.
        graph: Box<MatchNode>,
    },
}

// ── Match Token (prepared from CSS value) ──

/// A token prepared for matching against the match graph.
#[derive(Debug, Clone, PartialEq)]
pub struct MatchToken {
    /// Token type (e.g. `"ident-token"`, `"number-token"`).
    pub token_type: String,
    /// Token value as string.
    pub value: String,
}

// ── Match Result ──

/// Result of matching a CSS value against a definition syntax.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The matched token tree, if successful.
    pub matched: Option<Vec<MatchedItem>>,
    /// The error, if matching failed.
    pub error: Option<String>,
    /// Number of iterations taken by the matching algorithm.
    pub iterations: u32,
}

/// An item in the match result tree.
#[derive(Debug, Clone)]
pub enum MatchedItem {
    /// A matched CSS token.
    Token {
        /// Index into the token array.
        token_index: usize,
        /// The token value.
        value: String,
    },
    /// Open a syntax scope (Type or Property reference matched).
    OpenSyntax {
        /// Name of the syntax.
        name: String,
        /// Whether this is a Type or Property.
        kind: SyntaxKind,
    },
    /// Close a syntax scope.
    CloseSyntax {
        /// Name of the syntax being closed.
        name: String,
        /// Whether this is a Type or Property.
        kind: SyntaxKind,
    },
}

/// Whether a syntax reference is a Type or Property.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    /// A `<type>` reference.
    Type,
    /// A `<'property'>` reference.
    Property,
}

// ── Lexer Configuration ──

/// Configuration for the Lexer.
#[derive(Debug, Clone, Default)]
pub struct LexerConfig {
    /// Custom type definitions (name → syntax string).
    pub types: HashMap<String, String>,
    /// Custom property definitions (name → syntax string).
    pub properties: HashMap<String, String>,
    /// At-rule definitions.
    pub atrules: HashMap<String, AtruleConfig>,
    /// Whether built-in generic types are enabled.
    pub generic: bool,
}

/// Configuration for an at-rule.
#[derive(Debug, Clone, Default)]
pub struct AtruleConfig {
    /// Prelude syntax (if any).
    pub prelude: Option<String>,
    /// Descriptors (name → syntax string).
    pub descriptors: HashMap<String, String>,
}

// ── CSS-wide keywords ──

/// CSS-wide keywords that are always valid for any property.
pub const CSS_WIDE_KEYWORDS: &[&str] = &[
    "initial", "inherit", "unset", "revert", "revert-layer",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_node_terminal_states() {
        let m = MatchNode::Match;
        let mm = MatchNode::Mismatch;
        let de = MatchNode::DisallowEmpty;
        assert!(matches!(m, MatchNode::Match));
        assert!(matches!(mm, MatchNode::Mismatch));
        assert!(matches!(de, MatchNode::DisallowEmpty));
    }

    #[test]
    fn match_node_if_branch() {
        let node = MatchNode::If {
            condition: Box::new(MatchNode::Keyword { name: "auto".into() }),
            then_branch: Box::new(MatchNode::Match),
            else_branch: Box::new(MatchNode::Mismatch),
        };
        assert!(matches!(node, MatchNode::If { .. }));
    }

    #[test]
    fn match_node_enum() {
        let mut map = HashMap::new();
        map.insert("auto".into(), MatchNode::Match);
        map.insert("none".into(), MatchNode::Match);
        let node = MatchNode::Enum { map };
        if let MatchNode::Enum { map } = &node {
            assert_eq!(map.len(), 2);
        }
    }

    #[test]
    fn match_token_creation() {
        let token = MatchToken {
            token_type: "ident-token".into(),
            value: "red".into(),
        };
        assert_eq!(token.token_type, "ident-token");
        assert_eq!(token.value, "red");
    }

    #[test]
    fn match_result_success() {
        let result = MatchResult {
            matched: Some(vec![MatchedItem::Token {
                token_index: 0,
                value: "auto".into(),
            }]),
            error: None,
            iterations: 5,
        };
        assert!(result.matched.is_some());
        assert!(result.error.is_none());
    }

    #[test]
    fn match_result_failure() {
        let result = MatchResult {
            matched: None,
            error: Some("Mismatch".into()),
            iterations: 10,
        };
        assert!(result.matched.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn syntax_kind_type_vs_property() {
        assert_ne!(SyntaxKind::Type, SyntaxKind::Property);
    }

    #[test]
    fn lexer_config_default() {
        let config = LexerConfig::default();
        assert!(config.types.is_empty());
        assert!(config.properties.is_empty());
        assert!(!config.generic);
    }

    #[test]
    fn css_wide_keywords_count() {
        assert_eq!(CSS_WIDE_KEYWORDS.len(), 5);
        assert!(CSS_WIDE_KEYWORDS.contains(&"initial"));
        assert!(CSS_WIDE_KEYWORDS.contains(&"revert-layer"));
    }
}
