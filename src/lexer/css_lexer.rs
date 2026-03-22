//! Main Lexer struct for CSS value validation.

use std::collections::HashMap;

use super::error::SyntaxReferenceError;
use super::generic::get_generic_matcher;
use super::match_graph::build_match_graph;
use super::prepare_tokens::{prepare_tokens, PreparedToken};
use super::types::{AtruleConfig, LexerConfig, MatchNode, MatchResult, MatchedItem, CSS_WIDE_KEYWORDS};
use crate::definition_syntax::parse::parse as parse_definition_syntax;
use crate::tokenizer::types::TokenType;

/// Iteration limit to prevent infinite loops during matching.
const ITERATION_LIMIT: u32 = 150_000;

/// A syntax descriptor (lazy-parsed definition).
#[derive(Debug, Clone)]
struct SyntaxDescriptor {
    /// The raw syntax string.
    syntax: String,
    /// Cached match graph (built lazily).
    match_graph: Option<MatchNode>,
}

impl SyntaxDescriptor {
    fn new(syntax: &str) -> Self {
        Self {
            syntax: syntax.to_string(),
            match_graph: None,
        }
    }

    /// Get or build the match graph for this descriptor.
    fn get_match_graph(&mut self) -> Option<&MatchNode> {
        if self.match_graph.is_none() {
            if let Ok(ast) = parse_definition_syntax(&self.syntax) {
                self.match_graph = Some(build_match_graph(&ast));
            }
        }
        self.match_graph.as_ref()
    }
}

/// CSS Lexer — validates CSS values against definition syntax.
#[derive(Debug)]
#[allow(dead_code)] // atrules and generic will be used in later phases
pub struct Lexer {
    /// Type definitions (name → descriptor).
    types: HashMap<String, SyntaxDescriptor>,
    /// Property definitions (name → descriptor).
    properties: HashMap<String, SyntaxDescriptor>,
    /// At-rule definitions.
    atrules: HashMap<String, AtruleConfig>,
    /// Whether built-in generic types are enabled.
    generic: bool,
}

impl Lexer {
    /// Create a new Lexer with the given configuration.
    pub fn new(config: LexerConfig) -> Self {
        let mut types = HashMap::new();
        for (name, syntax) in &config.types {
            types.insert(name.clone(), SyntaxDescriptor::new(syntax));
        }

        let mut properties = HashMap::new();
        for (name, syntax) in &config.properties {
            properties.insert(name.clone(), SyntaxDescriptor::new(syntax));
        }

        Self {
            types,
            properties,
            atrules: config.atrules,
            generic: config.generic,
        }
    }

    /// Check if a property name is known.
    pub fn check_property_name(&self, name: &str) -> Result<(), SyntaxReferenceError> {
        let normalized = normalize_vendor_prefix(name);
        if self.properties.contains_key(normalized) || self.properties.contains_key(name) {
            Ok(())
        } else {
            Err(SyntaxReferenceError {
                message: format!("Unknown property `{name}`"),
                reference: name.to_string(),
            })
        }
    }

    /// Match a CSS value string against a property definition.
    pub fn match_property(&mut self, name: &str, value: &str) -> MatchResult {
        // Check for CSS-wide keywords
        let lower_value = value.trim().to_ascii_lowercase();
        if CSS_WIDE_KEYWORDS.contains(&lower_value.as_str()) {
            return MatchResult {
                matched: Some(vec![MatchedItem::Token {
                    token_index: 0,
                    value: value.to_string(),
                }]),
                error: None,
                iterations: 0,
            };
        }

        let normalized = normalize_vendor_prefix(name).to_string();
        let key = if self.properties.contains_key(&normalized) {
            normalized
        } else {
            name.to_string()
        };

        let graph = {
            let Some(desc) = self.properties.get_mut(&key) else {
                return MatchResult {
                    matched: None,
                    error: Some(format!("Unknown property `{name}`")),
                    iterations: 0,
                };
            };
            desc.get_match_graph().cloned()
        };

        match graph {
            Some(g) => {
                let tokens = prepare_tokens(value);
                self.match_tokens(&tokens, &g)
            }
            None => MatchResult {
                matched: None,
                error: Some(format!("Bad syntax for property `{name}`")),
                iterations: 0,
            },
        }
    }

    /// Match a CSS value string against a type definition.
    pub fn match_type(&mut self, name: &str, value: &str) -> MatchResult {
        let graph = {
            let Some(desc) = self.types.get_mut(name) else {
                return MatchResult {
                    matched: None,
                    error: Some(format!("Unknown type `{name}`")),
                    iterations: 0,
                };
            };
            desc.get_match_graph().cloned()
        };

        match graph {
            Some(g) => {
                let tokens = prepare_tokens(value);
                self.match_tokens(&tokens, &g)
            }
            None => MatchResult {
                matched: None,
                error: Some(format!("Bad syntax for type `{name}`")),
                iterations: 0,
            },
        }
    }

    /// Core matching algorithm: matches prepared tokens against a match graph.
    fn match_tokens(&self, tokens: &[PreparedToken], initial_state: &MatchNode) -> MatchResult {
        // Skip whitespace and comments
        let tokens: Vec<&PreparedToken> = tokens
            .iter()
            .filter(|t| t.token_type != TokenType::WhiteSpace && t.token_type != TokenType::Comment)
            .collect();

        if tokens.is_empty() {
            return MatchResult {
                matched: None,
                error: Some("Empty value".into()),
                iterations: 0,
            };
        }

        let mut token_index: usize = 0;
        let mut iterations: u32 = 0;
        let mut matched_tokens: Vec<MatchedItem> = Vec::new();

        // Simplified matching — handles the most common cases
        let result = self.match_recursive(
            &tokens,
            &mut token_index,
            initial_state,
            &mut matched_tokens,
            &mut iterations,
        );

        if result && token_index >= tokens.len() {
            MatchResult {
                matched: Some(matched_tokens),
                error: None,
                iterations,
            }
        } else {
            MatchResult {
                matched: None,
                error: Some("Mismatch".into()),
                iterations,
            }
        }
    }

    /// Recursive matching against a match node.
    #[allow(clippy::only_used_in_recursion, clippy::too_many_lines, clippy::self_only_used_in_recursion)]
    fn match_recursive(
        &self,
        tokens: &[&PreparedToken],
        token_index: &mut usize,
        node: &MatchNode,
        matched: &mut Vec<MatchedItem>,
        iterations: &mut u32,
    ) -> bool {
        *iterations += 1;
        if *iterations >= ITERATION_LIMIT {
            return false;
        }

        match node {
            MatchNode::Match => true,
            MatchNode::Mismatch => false,
            MatchNode::DisallowEmpty => !matched.is_empty(),

            MatchNode::If { condition, then_branch, else_branch } => {
                let saved_index = *token_index;
                let saved_len = matched.len();

                if self.match_recursive(tokens, token_index, condition, matched, iterations) {
                    self.match_recursive(tokens, token_index, then_branch, matched, iterations)
                } else {
                    *token_index = saved_index;
                    matched.truncate(saved_len);
                    self.match_recursive(tokens, token_index, else_branch, matched, iterations)
                }
            }

            MatchNode::Keyword { name } | MatchNode::AtKeyword { name } => {
                if *token_index < tokens.len() {
                    let token = tokens[*token_index];
                    if token.value.eq_ignore_ascii_case(name) {
                        matched.push(MatchedItem::Token {
                            token_index: *token_index,
                            value: token.value.clone(),
                        });
                        *token_index += 1;
                        return true;
                    }
                }
                false
            }

            MatchNode::Token { value } => {
                if *token_index < tokens.len() && tokens[*token_index].value == *value {
                    matched.push(MatchedItem::Token {
                        token_index: *token_index,
                        value: tokens[*token_index].value.clone(),
                    });
                    *token_index += 1;
                    return true;
                }
                false
            }

            MatchNode::Comma => {
                if *token_index < tokens.len() && tokens[*token_index].token_type == TokenType::Comma {
                    matched.push(MatchedItem::Token {
                        token_index: *token_index,
                        value: ",".into(),
                    });
                    *token_index += 1;
                    return true;
                }
                false
            }

            MatchNode::Generic { name } => {
                if *token_index >= tokens.len() {
                    return false;
                }
                if let Some(matcher) = get_generic_matcher(name) {
                    let token = tokens[*token_index];
                    let consumed = matcher(token.token_type, &token.value);
                    if consumed > 0 {
                        for i in 0..consumed {
                            if *token_index + i < tokens.len() {
                                matched.push(MatchedItem::Token {
                                    token_index: *token_index + i,
                                    value: tokens[*token_index + i].value.clone(),
                                });
                            }
                        }
                        *token_index += consumed;
                        return true;
                    }
                }
                false
            }

            MatchNode::Enum { map } => {
                if *token_index < tokens.len() {
                    let key = tokens[*token_index].value.to_ascii_lowercase();
                    if map.contains_key(&key) {
                        matched.push(MatchedItem::Token {
                            token_index: *token_index,
                            value: tokens[*token_index].value.clone(),
                        });
                        *token_index += 1;
                        return true;
                    }
                }
                false
            }

            MatchNode::Type { name } | MatchNode::Property { name } => {
                // For now, try to match using generic matchers for types
                if let MatchNode::Type { .. } = node {
                    if let Some(matcher) = get_generic_matcher(name) {
                        if *token_index < tokens.len() {
                            let token = tokens[*token_index];
                            let consumed = matcher(token.token_type, &token.value);
                            if consumed > 0 {
                                for i in 0..consumed {
                                    if *token_index + i < tokens.len() {
                                        matched.push(MatchedItem::Token {
                                            token_index: *token_index + i,
                                            value: tokens[*token_index + i].value.clone(),
                                        });
                                    }
                                }
                                *token_index += consumed;
                                return true;
                            }
                        }
                    }
                }
                false
            }

            MatchNode::Function { name } => {
                if *token_index < tokens.len() {
                    let token = tokens[*token_index];
                    if token.token_type == TokenType::Function
                        && token.value.eq_ignore_ascii_case(name)
                    {
                        matched.push(MatchedItem::Token {
                            token_index: *token_index,
                            value: token.value.clone(),
                        });
                        *token_index += 1;
                        return true;
                    }
                }
                false
            }

            MatchNode::StringMatch { value } => {
                let mut concat = String::new();
                let mut end_idx = *token_index;
                while end_idx < tokens.len() && concat.len() < value.len() {
                    concat.push_str(&tokens[end_idx].value);
                    end_idx += 1;
                }
                if concat.eq_ignore_ascii_case(value) {
                    for (i, tok) in tokens.iter().enumerate().take(end_idx).skip(*token_index) {
                        matched.push(MatchedItem::Token {
                            token_index: i,
                            value: tok.value.clone(),
                        });
                    }
                    *token_index = end_idx;
                    true
                } else {
                    false
                }
            }

            MatchNode::MatchGraph { graph } => {
                self.match_recursive(tokens, token_index, graph, matched, iterations)
            }

            MatchNode::MatchOnce { terms, all } => {
                let mut mask: u32 = 0;
                let all_mask = (1u32 << terms.len()) - 1;

                loop {
                    let mut any_matched = false;
                    for (i, term) in terms.iter().enumerate() {
                        let flag = 1u32 << i;
                        if mask & flag != 0 {
                            continue;
                        }

                        let saved_index = *token_index;
                        let saved_len = matched.len();

                        if self.match_recursive(tokens, token_index, term, matched, iterations) {
                            mask |= flag;
                            any_matched = true;
                            break;
                        }
                        *token_index = saved_index;
                        matched.truncate(saved_len);
                    }

                    if !any_matched {
                        break;
                    }

                    if mask == all_mask {
                        return true;
                    }
                }

                if *all {
                    mask == all_mask
                } else {
                    mask != 0
                }
            }
        }
    }
}

/// Strip vendor prefix from a property name (e.g., `-webkit-transform` → `transform`).
fn normalize_vendor_prefix(name: &str) -> &str {
    if let Some(stripped) = name.strip_prefix('-') {
        // Find second hyphen after vendor prefix
        if let Some(pos) = stripped.find('-') {
            return &stripped[pos + 1..];
        }
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_lexer() -> Lexer {
        let mut config = LexerConfig::default();
        config.generic = true;
        config.properties.insert(
            "color".into(),
            "<color> | auto".into(),
        );
        config.properties.insert(
            "display".into(),
            "block | inline | none | flex | grid".into(),
        );
        config.properties.insert(
            "margin".into(),
            "<length> | <percentage> | auto".into(),
        );
        config.types.insert(
            "color".into(),
            "<hex-color> | <ident>".into(),
        );
        Lexer::new(config)
    }

    #[test]
    fn match_keyword_property() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("display", "block");
        assert!(result.matched.is_some(), "Expected match for 'display: block'");
    }

    #[test]
    fn match_keyword_none() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("display", "none");
        assert!(result.matched.is_some());
    }

    #[test]
    fn mismatch_unknown_keyword() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("display", "banana");
        assert!(result.matched.is_none());
    }

    #[test]
    fn match_css_wide_keyword() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("display", "initial");
        assert!(result.matched.is_some());
    }

    #[test]
    fn match_css_wide_inherit() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("display", "inherit");
        assert!(result.matched.is_some());
    }

    #[test]
    fn unknown_property() {
        let mut lexer = make_lexer();
        let result = lexer.match_property("nonexistent", "value");
        assert!(result.matched.is_none());
        assert!(result.error.is_some());
    }

    #[test]
    fn check_property_name_known() {
        let lexer = make_lexer();
        assert!(lexer.check_property_name("display").is_ok());
    }

    #[test]
    fn check_property_name_unknown() {
        let lexer = make_lexer();
        assert!(lexer.check_property_name("nonexistent").is_err());
    }

    #[test]
    fn normalize_vendor_prefix_works() {
        assert_eq!(normalize_vendor_prefix("-webkit-transform"), "transform");
        assert_eq!(normalize_vendor_prefix("-moz-appearance"), "appearance");
        assert_eq!(normalize_vendor_prefix("color"), "color");
    }
}
