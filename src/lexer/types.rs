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

    /// Repeating match: match `term` between `min` and `max` times.
    /// If `comma` is true, elements are comma-separated.
    Repeat {
        /// The term to repeat.
        term: Box<MatchNode>,
        /// Minimum repetitions (0 for *, 1 for +/#).
        min: u32,
        /// Maximum repetitions (`u32::MAX` for unbounded).
        max: u32,
        /// Whether repetitions are comma-separated (#).
        comma: bool,
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

/// A trace entry showing how a match resolved through the syntax tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceEntry {
    /// Whether this was a Type or Property.
    pub kind: SyntaxKind,
    /// Name of the syntax.
    pub name: String,
}

impl MatchResult {
    /// Get the trace path from root syntax to the matched node at `token_index`.
    pub fn get_trace(&self, token_index: usize) -> Option<Vec<TraceEntry>> {
        let items = self.matched.as_ref()?;
        let mut trace = Vec::new();
        let mut stack: Vec<TraceEntry> = Vec::new();

        for item in items {
            match item {
                MatchedItem::OpenSyntax { name, kind } => {
                    stack.push(TraceEntry { kind: *kind, name: name.clone() });
                }
                MatchedItem::CloseSyntax { .. } => {
                    stack.pop();
                }
                MatchedItem::Token { token_index: idx, .. } if *idx == token_index => {
                    trace.clone_from(&stack);
                    break;
                }
                MatchedItem::Token { .. } => {}
            }
        }

        if trace.is_empty() && !stack.is_empty() {
            return None;
        }
        Some(trace)
    }

    /// Check if the matched node at `token_index` is inside a Type with the given name.
    pub fn is_type(&self, token_index: usize, type_name: &str) -> bool {
        self.get_trace(token_index)
            .is_some_and(|trace| trace.iter().any(|e| e.kind == SyntaxKind::Type && e.name == type_name))
    }

    /// Check if the matched node at `token_index` is inside a Property with the given name.
    pub fn is_property(&self, token_index: usize, property_name: &str) -> bool {
        self.get_trace(token_index)
            .is_some_and(|trace| trace.iter().any(|e| e.kind == SyntaxKind::Property && e.name == property_name))
    }

    /// Check if the matched node at `token_index` was matched as a Keyword.
    pub fn is_keyword(&self, token_index: usize) -> bool {
        // A token is a keyword if it's inside an OpenSyntax with kind Keyword
        // In practice, we check if the matched items contain a keyword marker for this index
        let Some(items) = &self.matched else { return false };
        // Look for a Keyword open/close around this token
        let mut in_keyword = false;
        for item in items {
            match item {
                MatchedItem::OpenSyntax { kind: SyntaxKind::Keyword, .. } => in_keyword = true,
                MatchedItem::CloseSyntax { kind: SyntaxKind::Keyword, .. } => in_keyword = false,
                MatchedItem::Token { token_index: idx, .. } if *idx == token_index => {
                    return in_keyword;
                }
                _ => {}
            }
        }
        false
    }
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

/// Whether a syntax reference is a Type, Property, or Keyword.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxKind {
    /// A `<type>` reference.
    Type,
    /// A `<'property'>` reference.
    Property,
    /// A keyword match.
    Keyword,
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
    /// Override CSS-wide keywords (default: initial, inherit, unset, revert, revert-layer).
    pub css_wide_keywords: Option<Vec<String>>,
    /// Override unit groups (only existing groups are overridden, new groups ignored).
    pub units: HashMap<String, Vec<String>>,
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

/// Result of calling `Lexer::validate()`.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Error messages for broken definitions.
    pub errors: Vec<String>,
    /// Type names with broken definitions.
    pub types: Vec<String>,
    /// Property names with broken definitions.
    pub properties: Vec<String>,
}

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
