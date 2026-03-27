//! Main Lexer struct for CSS value validation.

use std::cell::RefCell;
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
#[derive(Debug)]
struct SyntaxDescriptor {
    /// The raw syntax string.
    syntax: String,
    /// Cached match graph (built lazily via interior mutability).
    match_graph: RefCell<Option<MatchNode>>,
}

impl Clone for SyntaxDescriptor {
    fn clone(&self) -> Self {
        Self {
            syntax: self.syntax.clone(),
            match_graph: RefCell::new(self.match_graph.borrow().clone()),
        }
    }
}

impl SyntaxDescriptor {
    fn new(syntax: &str) -> Self {
        Self {
            syntax: syntax.to_string(),
            match_graph: RefCell::new(None),
        }
    }

    /// Get or build the match graph for this descriptor (uses interior mutability).
    fn get_match_graph(&self) -> Option<MatchNode> {
        {
            let cached = self.match_graph.borrow();
            if cached.is_some() {
                return cached.clone();
            }
        }
        if let Ok(ast) = parse_definition_syntax(&self.syntax) {
            let graph = build_match_graph(&ast);
            *self.match_graph.borrow_mut() = Some(graph.clone());
            Some(graph)
        } else {
            None
        }
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
    pub fn match_property(&self, name: &str, value: &str) -> MatchResult {
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
            let Some(desc) = self.properties.get(&key) else {
                return MatchResult {
                    matched: None,
                    error: Some(format!("Unknown property `{name}`")),
                    iterations: 0,
                };
            };
            desc.get_match_graph()
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
    pub fn match_type(&self, name: &str, value: &str) -> MatchResult {
        let graph = {
            let Some(desc) = self.types.get(name) else {
                return MatchResult {
                    matched: None,
                    error: Some(format!("Unknown type `{name}`")),
                    iterations: 0,
                };
            };
            desc.get_match_graph()
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
                let is_type = matches!(node, MatchNode::Type { .. });

                // First try generic matchers for types
                if is_type {
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

                // Then try resolving from the types/properties map
                let graph = if is_type {
                    self.types.get(name).and_then(SyntaxDescriptor::get_match_graph)
                } else {
                    self.properties.get(name).and_then(SyntaxDescriptor::get_match_graph)
                };

                if let Some(g) = graph {
                    let saved_index = *token_index;
                    let saved_len = matched.len();
                    if self.match_recursive(tokens, token_index, &g, matched, iterations) {
                        return true;
                    }
                    *token_index = saved_index;
                    matched.truncate(saved_len);
                }

                false
            }

            MatchNode::Function { name } => {
                if *token_index < tokens.len() {
                    let token = tokens[*token_index];
                    if token.token_type == TokenType::Function {
                        // Token value includes trailing '(' (e.g., "rgb("), strip it for comparison
                        let func_name = token.value.strip_suffix('(').unwrap_or(&token.value);
                        if func_name.eq_ignore_ascii_case(name) {
                            matched.push(MatchedItem::Token {
                                token_index: *token_index,
                                value: token.value.clone(),
                            });
                            *token_index += 1;
                            return true;
                        }
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

    /// Register a new property definition.
    pub fn add_property(&mut self, name: &str, syntax: &str) {
        self.properties.insert(name.to_string(), SyntaxDescriptor::new(syntax));
    }

    /// Register a new type definition.
    pub fn add_type(&mut self, name: &str, syntax: &str) {
        self.types.insert(name.to_string(), SyntaxDescriptor::new(syntax));
    }

    /// Get all registered property names.
    pub fn property_names(&self) -> Vec<&str> {
        self.properties.keys().map(String::as_str).collect()
    }

    /// Get all registered type names.
    pub fn type_names(&self) -> Vec<&str> {
        self.types.keys().map(String::as_str).collect()
    }

    // ── At-rule methods ──

    /// Resolve an at-rule name, handling case-insensitivity and vendor prefixes.
    /// Returns the key under which the atrule is stored, or None if not found.
    fn get_atrule_key(&self, name: &str) -> Option<String> {
        let lower = name.to_ascii_lowercase();
        // Try exact (lowercased) name first
        if self.atrules.contains_key(&lower) {
            return Some(lower);
        }
        // Try stripping vendor prefix
        let basename = normalize_vendor_prefix(&lower).to_string();
        if basename != lower && self.atrules.contains_key(&basename) {
            return Some(basename);
        }
        None
    }

    /// Check if an at-rule name is known.
    pub fn check_atrule_name(&self, name: &str) -> Result<(), SyntaxReferenceError> {
        if self.get_atrule_key(name).is_some() {
            Ok(())
        } else {
            Err(SyntaxReferenceError {
                message: format!("Unknown at-rule `@{name}`"),
                reference: format!("@{name}"),
            })
        }
    }

    /// Check if an at-rule prelude is valid.
    /// `prelude` of `None` or `Some("")` means no prelude was provided.
    pub fn check_atrule_prelude(
        &self,
        name: &str,
        prelude: Option<&str>,
    ) -> Result<(), SyntaxReferenceError> {
        self.check_atrule_name(name)?;

        let key = self.get_atrule_key(name).unwrap();
        let atrule = &self.atrules[&key];
        let has_prelude_syntax = atrule.prelude.is_some();
        let prelude_is_empty = prelude.is_none() || prelude == Some("");

        if !has_prelude_syntax && !prelude_is_empty {
            return Err(SyntaxReferenceError {
                message: format!("At-rule `@{name}` should not contain a prelude"),
                reference: format!("@{name}"),
            });
        }

        if has_prelude_syntax && prelude_is_empty {
            // Check if the syntax allows an empty match
            let syntax_str = atrule.prelude.clone().unwrap();
            let result = self.match_syntax_str(&syntax_str, "");
            if result.matched.is_none() {
                return Err(SyntaxReferenceError {
                    message: format!("At-rule `@{name}` should contain a prelude"),
                    reference: format!("@{name}"),
                });
            }
        }

        Ok(())
    }

    /// Check if a descriptor name is valid for an at-rule.
    pub fn check_atrule_descriptor_name(
        &self,
        atrule_name: &str,
        descriptor_name: Option<&str>,
    ) -> Result<(), SyntaxReferenceError> {
        self.check_atrule_name(atrule_name)?;

        let key = self.get_atrule_key(atrule_name).unwrap();
        let atrule = &self.atrules[&key];

        if atrule.descriptors.is_empty() {
            return Err(SyntaxReferenceError {
                message: format!("At-rule `@{atrule_name}` has no known descriptors"),
                reference: format!("@{atrule_name}"),
            });
        }

        if let Some(desc_name) = descriptor_name {
            let lower_desc = desc_name.to_ascii_lowercase();
            let basename = normalize_vendor_prefix(&lower_desc).to_string();
            if !atrule.descriptors.contains_key(&lower_desc)
                && !atrule.descriptors.contains_key(&basename)
            {
                return Err(SyntaxReferenceError {
                    message: format!("Unknown at-rule descriptor `{desc_name}`"),
                    reference: desc_name.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Match an at-rule prelude against its syntax definition.
    pub fn match_atrule_prelude(
        &self,
        name: &str,
        prelude: Option<&str>,
    ) -> MatchResult {
        if let Err(e) = self.check_atrule_prelude(name, prelude) {
            return MatchResult {
                matched: None,
                error: Some(e.message),
                iterations: 0,
            };
        }

        let key = self.get_atrule_key(name).unwrap();
        let atrule = &self.atrules[&key];

        if atrule.prelude.is_none() {
            // No prelude syntax defined — positive result with no match data
            return MatchResult {
                matched: None,
                error: None,
                iterations: 0,
            };
        }

        let syntax_str = atrule.prelude.clone().unwrap();
        let value = prelude.unwrap_or("");

        if value.is_empty() {
            // Empty prelude matched the syntax check (allowed empty) —
            // return an empty match
            return MatchResult {
                matched: Some(vec![]),
                error: None,
                iterations: 0,
            };
        }

        self.match_syntax_str(&syntax_str, value)
    }

    /// Match an at-rule descriptor value against its syntax definition.
    pub fn match_atrule_descriptor(
        &self,
        atrule_name: &str,
        descriptor_name: &str,
        value: &str,
    ) -> MatchResult {
        if let Err(e) = self.check_atrule_descriptor_name(atrule_name, Some(descriptor_name)) {
            return MatchResult {
                matched: None,
                error: Some(e.message),
                iterations: 0,
            };
        }

        // Reject CSS-wide keywords for descriptors
        let lower_value = value.trim().to_ascii_lowercase();
        if CSS_WIDE_KEYWORDS.contains(&lower_value.as_str()) {
            return MatchResult {
                matched: None,
                error: Some("Mismatch".into()),
                iterations: 0,
            };
        }

        let key = self.get_atrule_key(atrule_name).unwrap();
        let atrule = &self.atrules[&key];

        // Resolve descriptor: try vendor-prefixed name first, then basename
        let lower_desc = descriptor_name.to_ascii_lowercase();
        let basename = normalize_vendor_prefix(&lower_desc).to_string();
        let syntax_str = atrule
            .descriptors
            .get(&lower_desc)
            .or_else(|| atrule.descriptors.get(&basename))
            .cloned()
            .unwrap();

        self.match_syntax_str(&syntax_str, value)
    }

    /// Match a value string against a raw syntax definition string.
    fn match_syntax_str(&self, syntax: &str, value: &str) -> MatchResult {
        let graph = {
            if let Ok(ast) = parse_definition_syntax(syntax) {
                Some(build_match_graph(&ast))
            } else {
                None
            }
        };

        match graph {
            Some(g) => {
                let tokens = prepare_tokens(value);
                // For empty values, check if the graph can match with no tokens
                let filtered: Vec<&PreparedToken> = tokens
                    .iter()
                    .filter(|t| {
                        t.token_type != TokenType::WhiteSpace
                            && t.token_type != TokenType::Comment
                    })
                    .collect();
                if filtered.is_empty() {
                    return self.match_empty(&g);
                }
                self.match_tokens(&tokens, &g)
            }
            None => MatchResult {
                matched: None,
                error: Some(format!("Bad syntax: {syntax}")),
                iterations: 0,
            },
        }
    }

    /// Check if a match graph can match with zero tokens (e.g., optional syntax).
    fn match_empty(&self, node: &MatchNode) -> MatchResult {
        // Walk the graph to see if we can reach Match without consuming tokens
        if self.can_match_empty(node) {
            MatchResult {
                matched: Some(vec![]),
                error: None,
                iterations: 0,
            }
        } else {
            MatchResult {
                matched: None,
                error: Some("Mismatch".into()),
                iterations: 0,
            }
        }
    }

    /// Check if a match graph node can succeed without consuming any tokens.
    #[allow(clippy::self_only_used_in_recursion)]
    fn can_match_empty(&self, node: &MatchNode) -> bool {
        match node {
            MatchNode::Match => true,
            MatchNode::If { condition, then_branch, else_branch } => {
                if self.can_match_empty(condition) {
                    self.can_match_empty(then_branch)
                } else {
                    self.can_match_empty(else_branch)
                }
            }
            MatchNode::MatchGraph { graph } => self.can_match_empty(graph),
            // All other nodes require at least one token
            _ => false,
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
        let lexer = make_lexer();
        let result = lexer.match_property("display", "block");
        assert!(result.matched.is_some(), "Expected match for 'display: block'");
    }

    #[test]
    fn match_keyword_none() {
        let lexer = make_lexer();
        let result = lexer.match_property("display", "none");
        assert!(result.matched.is_some());
    }

    #[test]
    fn mismatch_unknown_keyword() {
        let lexer = make_lexer();
        let result = lexer.match_property("display", "banana");
        assert!(result.matched.is_none());
    }

    #[test]
    fn match_css_wide_keyword() {
        let lexer = make_lexer();
        let result = lexer.match_property("display", "initial");
        assert!(result.matched.is_some());
    }

    #[test]
    fn match_css_wide_inherit() {
        let lexer = make_lexer();
        let result = lexer.match_property("display", "inherit");
        assert!(result.matched.is_some());
    }

    #[test]
    fn unknown_property() {
        let lexer = make_lexer();
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
