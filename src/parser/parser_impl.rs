//! Core parser implementation.
//!
//! Parser uses closures extensively for the recognizer/fallback pattern.
//! Many clippy pedantic lints conflict with this architecture.
#![allow(
    clippy::redundant_closure,
    clippy::redundant_closure_for_method_calls,
    clippy::needless_pass_by_value,
    clippy::unnecessary_wraps,
    clippy::match_same_arms,
    clippy::wildcard_imports,
    clippy::bool_to_int_with_if,
    clippy::single_match_else,
    clippy::unused_self,
    clippy::needless_lifetimes,
    clippy::doc_markdown,
    clippy::if_then_some_else_none,
)]

use crate::ast::*;
use crate::tokenizer::offset_to_location::OffsetToLocation;
use crate::tokenizer::token_stream::TokenStream;
use crate::tokenizer::types::TokenType;

use super::error::CssSyntaxError;
use super::options::{ParseFlags, ParseOptions};

/// CSS recursive descent parser.
#[derive(Debug)]
pub struct Parser {
    /// Token stream for navigation.
    pub stream: TokenStream,
    /// Parse behavior flags.
    pub flags: ParseFlags,
    /// Source filename.
    pub filename: Option<String>,
    /// Offset-to-location mapper (lazy).
    pub locator: OffsetToLocation,
}

impl Parser {
    /// Create a parser for the given source string.
    pub fn new(source: &str, options: &ParseOptions) -> Self {
        let locator = OffsetToLocation::new(source, options.offset, options.line, options.column);
        Self {
            stream: TokenStream::new(source),
            flags: options.flags.clone(),
            filename: options.filename.clone(),
            locator,
        }
    }

    // ── Location helpers ──

    fn loc_start(&mut self) -> Option<Position> {
        if !self.flags.positions {
            return None;
        }
        let loc = self.locator.get_location(self.stream.token_start, None);
        Some(Position {
            offset: loc.offset,
            line: loc.line,
            column: loc.column,
        })
    }

    fn loc_end(&mut self) -> Option<Position> {
        if !self.flags.positions {
            return None;
        }
        let loc = self.locator.get_location(self.stream.token_end, None);
        Some(Position {
            offset: loc.offset,
            line: loc.line,
            column: loc.column,
        })
    }

    fn make_loc(&mut self, start: Option<Position>) -> Option<Loc> {
        start.map(|s| {
            let end = self.loc_end().unwrap_or_else(|| s.clone());
            Loc {
                source: self.filename.clone(),
                start: s,
                end,
            }
        })
    }

    // ── Token helpers ──

    fn token_type(&self) -> TokenType {
        self.stream.token_type
    }

    fn source(&self) -> &str {
        self.stream.source()
    }

    fn token_value(&self) -> &str {
        &self.stream.source()[self.stream.token_start..self.stream.token_end]
    }

    fn next(&mut self) {
        self.stream.next();
    }

    fn skip_sc(&mut self) {
        self.stream.skip_sc();
    }

    fn eat(&mut self, expected: TokenType) -> Result<(), CssSyntaxError> {
        if self.token_type() != expected {
            return Err(self.error(format!(
                "Expected {}, got {}",
                expected.as_spec_name(),
                self.token_type().as_spec_name()
            )));
        }
        self.next();
        Ok(())
    }

    fn error(&mut self, message: String) -> CssSyntaxError {
        let loc = self.locator.get_location(self.stream.token_start, None);
        CssSyntaxError::new(message, self.source().to_string(), self.stream.token_start, loc.line, loc.column)
    }

    // ── Core: read_sequence ──

    fn read_sequence(&mut self, get_node: impl Fn(&mut Self) -> Option<Node>, on_whitespace: impl Fn(&mut Self, Option<&Node>, &mut Vec<Node>)) -> Vec<Node> {
        let mut children = Vec::new();
        let mut has_space = false;

        while !self.stream.eof {
            match self.token_type() {
                TokenType::WhiteSpace => {
                    has_space = true;
                    self.next();
                    continue;
                }
                TokenType::Comment => {
                    self.next();
                    continue;
                }
                _ => {}
            }

            if let Some(child) = get_node(self) {
                if has_space {
                    on_whitespace(self, Some(&child), &mut children);
                    has_space = false;
                }
                children.push(child);
            } else {
                break;
            }
        }

        if has_space {
            on_whitespace(self, None, &mut children);
        }

        children
    }

    // ── Error recovery ──

    fn parse_with_fallback(&mut self, consumer: impl FnOnce(&mut Self) -> Result<Node, CssSyntaxError>, fallback: impl FnOnce(&mut Self) -> Node) -> Node {
        let start_index = self.stream.token_index();
        match consumer(self) {
            Ok(node) => node,
            Err(_) => {
                // Rewind to start
                let current = self.stream.token_index();
                if current > start_index {
                    // Reset by re-creating navigation state
                    self.stream.reset();
                    self.stream.next();
                    for _ in 0..start_index {
                        self.stream.next();
                    }
                }
                fallback(self)
            }
        }
    }

    // ── Raw node (error recovery) ──

    fn consume_raw(&mut self, stop: impl Fn(u8) -> u8) -> Node {
        let start = self.loc_start();
        let start_offset = self.stream.token_start;
        let start_token = self.stream.token_index();

        self.stream.skip_until_balanced(start_token, stop);

        let value = self.source()[start_offset..self.stream.token_start].to_string();
        Node::Raw(Raw {
            loc: self.make_loc(start),
            value,
        })
    }

    // ── Node parse functions ──

    /// Parse a `StyleSheet` node.
    pub fn parse_stylesheet(&mut self) -> Node {
        let start = self.loc_start();
        let mut children = Vec::new();

        while !self.stream.eof {
            match self.token_type() {
                TokenType::WhiteSpace | TokenType::Comment => {
                    self.next();
                }
                TokenType::Cdo => {
                    children.push(self.parse_cdo());
                }
                TokenType::Cdc => {
                    children.push(self.parse_cdc());
                }
                TokenType::AtKeyword => {
                    let node = self.parse_with_fallback(
                        |p| p.parse_atrule_result(),
                        |p| p.consume_raw(|_| 0),
                    );
                    children.push(node);
                }
                _ => {
                    let node = self.parse_with_fallback(
                        |p| p.parse_rule_result(),
                        |p| p.consume_raw(|_| 0),
                    );
                    children.push(node);
                }
            }
        }

        Node::StyleSheet(StyleSheet {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `Rule` node.
    pub fn parse_rule(&mut self) -> Node {
        let start = self.loc_start();

        let prelude = if self.flags.parse_rule_prelude {
            self.parse_with_fallback(
                |p| Ok(p.parse_selector_list()),
                |p| p.consume_raw(|code| if code == 0x7B { 1 } else { 0 }),
            )
        } else {
            self.consume_raw(|code| if code == 0x7B { 1 } else { 0 })
        };

        let block = self.parse_block(true);

        Node::Rule(Rule {
            loc: self.make_loc(start),
            prelude: Box::new(prelude),
            block: Box::new(block),
        })
    }

    fn parse_rule_result(&mut self) -> Result<Node, CssSyntaxError> {
        Ok(self.parse_rule())
    }

    /// Parse a `Declaration` node.
    pub fn parse_declaration(&mut self) -> Result<Node, CssSyntaxError> {
        let start = self.loc_start();

        // Read property name (may include hacks like *, $, +)
        let property = self.read_property();

        self.skip_sc();
        self.eat(TokenType::Colon)?;
        self.skip_sc();

        let is_custom = property.starts_with("--");

        let value = if self.flags.parse_value && !is_custom {
            self.parse_value()
        } else {
            self.consume_raw(|code| {
                if code == 0x21 || code == 0x3B { 1 } else { 0 }
            })
        };

        let important = self.parse_important();

        Node::Declaration(Declaration {
            loc: self.make_loc(start),
            important,
            property,
            value: Box::new(value),
        })
        .pipe_ok()
    }

    fn read_property(&mut self) -> String {
        let start = self.stream.token_start;
        // Handle hack prefixes: *, $, +, #, &, /
        if self.token_type() == TokenType::Delim {
            self.next();
            if self.token_type() == TokenType::Delim {
                // // double slash hack
                self.next();
            }
        }
        if self.token_type() == TokenType::Ident || self.token_type() == TokenType::Hash {
            let end = self.stream.token_end;
            self.next();
            self.source()[start..end].to_string()
        } else {
            self.source()[start..self.stream.token_end].to_string()
        }
    }

    fn parse_important(&mut self) -> bool {
        // Check for !important
        if self.token_type() == TokenType::Delim
            && self.stream.source().as_bytes().get(self.stream.token_start) == Some(&b'!')
        {
            self.next();
            self.skip_sc();
            if self.token_type() == TokenType::Ident {
                let val = self.token_value();
                if val.eq_ignore_ascii_case("important") {
                    self.next();
                    return true;
                }
            }
        }
        false
    }

    /// Parse a `SelectorList` node.
    pub fn parse_selector_list(&mut self) -> Node {
        let start = self.loc_start();
        let mut children = Vec::new();

        children.push(self.parse_selector());

        while self.token_type() == TokenType::Comma {
            self.next();
            children.push(self.parse_selector());
        }

        Node::SelectorList(SelectorList {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `Selector` node.
    pub fn parse_selector(&mut self) -> Node {
        let start = self.loc_start();
        let children = self.read_sequence(
            |p| p.selector_get_node(),
            |p, next, children| p.selector_on_whitespace(next, children),
        );

        Node::Selector(Selector {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `Value` node.
    pub fn parse_value(&mut self) -> Node {
        let start = self.loc_start();
        let children = self.read_sequence(
            |p| p.value_get_node(),
            |_p, _next, _children| {},
        );

        Node::Value(Value {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `Block` node.
    pub fn parse_block(&mut self, is_style_block: bool) -> Node {
        let start = self.loc_start();
        let _ = self.eat(TokenType::LeftCurlyBracket);
        let mut children = Vec::new();

        while !self.stream.eof && self.token_type() != TokenType::RightCurlyBracket {
            match self.token_type() {
                TokenType::WhiteSpace | TokenType::Comment => {
                    self.next();
                }
                TokenType::AtKeyword => {
                    let node = self.parse_with_fallback(
                        |p| p.parse_atrule_result(),
                        |p| p.consume_raw(|_| 0),
                    );
                    children.push(node);
                }
                TokenType::Semicolon => {
                    self.next();
                }
                _ => {
                    if is_style_block {
                        let node = self.parse_with_fallback(
                            |p| p.parse_declaration(),
                            |p| p.consume_raw(|code| if code == 0x3B { 2 } else { 0 }),
                        );
                        children.push(node);
                    } else {
                        let node = self.parse_with_fallback(
                            |p| p.parse_rule_result(),
                            |p| p.consume_raw(|_| 0),
                        );
                        children.push(node);
                    }
                }
            }
        }

        if self.token_type() == TokenType::RightCurlyBracket {
            self.next();
        }

        Node::Block(Block {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse an `Atrule` node.
    pub fn parse_atrule(&mut self) -> Node {
        let start = self.loc_start();
        let name = self.token_value()[1..].to_string(); // strip @
        self.next();

        // Parse prelude (everything until { or ;)
        let prelude = if self.token_type() != TokenType::LeftCurlyBracket
            && self.token_type() != TokenType::Semicolon
            && !self.stream.eof
        {
            Some(Box::new(if self.flags.parse_atrule_prelude {
                self.parse_atrule_prelude()
            } else {
                self.consume_raw(|code| if code == 0x7B || code == 0x3B { 1 } else { 0 })
            }))
        } else {
            None
        };

        // Parse block or consume semicolon
        let block = if self.token_type() == TokenType::LeftCurlyBracket {
            Some(Box::new(self.parse_block(is_style_atrule(&name))))
        } else {
            if self.token_type() == TokenType::Semicolon {
                self.next();
            }
            None
        };

        Node::Atrule(Atrule {
            loc: self.make_loc(start),
            name,
            prelude,
            block,
        })
    }

    fn parse_atrule_result(&mut self) -> Result<Node, CssSyntaxError> {
        Ok(self.parse_atrule())
    }

    fn parse_atrule_prelude(&mut self) -> Node {
        let start = self.loc_start();
        let children = self.read_sequence(
            |p| p.value_get_node(),
            |_p, _next, _children| {},
        );
        Node::AtrulePrelude(AtrulePrelude {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `MediaQueryList` node (comma-separated media queries).
    pub fn parse_media_query_list(&mut self) -> Node {
        let start = self.loc_start();
        let mut children = Vec::new();

        children.push(self.parse_media_query());

        while self.token_type() == TokenType::Comma {
            self.next();
            self.skip_sc();
            children.push(self.parse_media_query());
        }

        Node::MediaQueryList(MediaQueryList {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `MediaQuery` node.
    pub fn parse_media_query(&mut self) -> Node {
        let start = self.loc_start();
        self.skip_sc();

        let mut modifier = None;
        let mut media_type = None;
        let mut condition = None;

        // Check for modifier (not/only) or media type
        if self.token_type() == TokenType::Ident {
            let val = self.token_value().to_ascii_lowercase();
            match val.as_str() {
                "not" | "only" => {
                    modifier = Some(val);
                    self.next();
                    self.skip_sc();
                    if self.token_type() == TokenType::Ident {
                        media_type = Some(self.token_value().to_string());
                        self.next();
                        self.skip_sc();
                    }
                }
                _ => {
                    // Could be a media type or condition start
                    media_type = Some(val);
                    self.next();
                    self.skip_sc();
                }
            }
        }

        // Parse condition if present (after "and" keyword or standalone)
        if self.token_type() == TokenType::Ident {
            let kw = self.token_value().to_ascii_lowercase();
            if kw == "and" || kw == "or" || kw == "not" {
                condition = Some(Box::new(self.parse_condition("media")));
            }
        } else if self.token_type() == TokenType::LeftParenthesis {
            // Condition without media type
            if media_type.is_some() && modifier.is_none() {
                // Was actually a condition start, not a media type
                // Reparse — simplified: treat as-is for now
            }
            condition = Some(Box::new(self.parse_condition("media")));
        }

        Node::MediaQuery(MediaQuery {
            loc: self.make_loc(start),
            modifier,
            media_type,
            condition,
        })
    }

    /// Parse a `Condition` node (media or supports condition).
    pub fn parse_condition(&mut self, kind: &str) -> Node {
        let start = self.loc_start();
        let mut children = Vec::new();

        // Consume condition terms and combinators (and/or/not)
        while !self.stream.eof
            && self.token_type() != TokenType::LeftCurlyBracket
            && self.token_type() != TokenType::Semicolon
            && self.token_type() != TokenType::RightParenthesis
        {
            match self.token_type() {
                TokenType::WhiteSpace | TokenType::Comment => {
                    self.next();
                }
                TokenType::Ident => {
                    children.push(self.parse_identifier());
                }
                TokenType::LeftParenthesis => {
                    children.push(self.parse_parentheses());
                }
                TokenType::Function => {
                    children.push(self.parse_function());
                }
                _ => break,
            }
        }

        Node::Condition(Condition {
            loc: self.make_loc(start),
            kind: kind.to_string(),
            children,
        })
    }

    /// Parse a `LayerList` node.
    pub fn parse_layer_list(&mut self) -> Node {
        let start = self.loc_start();
        let mut children = Vec::new();

        children.push(self.parse_layer());

        while self.token_type() == TokenType::Comma {
            self.next();
            self.skip_sc();
            children.push(self.parse_layer());
        }

        Node::LayerList(LayerList {
            loc: self.make_loc(start),
            children,
        })
    }

    /// Parse a `Layer` node.
    pub fn parse_layer(&mut self) -> Node {
        let start = self.loc_start();
        self.skip_sc();
        let mut name = String::new();

        // Layer name can be dotted: a.b.c
        while self.token_type() == TokenType::Ident {
            if !name.is_empty() {
                name.push('.');
            }
            name.push_str(self.token_value());
            self.next();
            if self.token_type() == TokenType::Delim
                && self.source().as_bytes().get(self.stream.token_start) == Some(&b'.')
            {
                self.next();
            } else {
                break;
            }
        }

        Node::Layer(Layer {
            loc: self.make_loc(start),
            name,
        })
    }

    /// Parse a `Comment` node.
    pub fn parse_comment(&mut self) -> Node {
        let start = self.loc_start();
        let raw = self.token_value();
        // Strip /* and */
        let value = if raw.len() >= 4 {
            raw[2..raw.len() - 2].to_string()
        } else {
            raw.to_string()
        };
        self.next();
        Node::Comment(Comment {
            loc: self.make_loc(start),
            value,
        })
    }

    /// Parse a `WhiteSpace` node.
    pub fn parse_whitespace(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value().to_string();
        self.next();
        Node::WhiteSpace(WhiteSpace {
            loc: self.make_loc(start),
            value,
        })
    }

    // ── Selector scope node recognizer ──

    fn selector_get_node(&mut self) -> Option<Node> {
        match self.token_type() {
            TokenType::LeftSquareBracket => Some(self.parse_attribute_selector()),
            TokenType::Hash => Some(self.parse_id_selector()),
            TokenType::Ident => Some(self.parse_type_selector()),
            TokenType::Colon => {
                // Check for :: (pseudo-element) vs : (pseudo-class)
                if self.stream.lookup_type(1) == TokenType::Colon {
                    Some(self.parse_pseudo_element_selector())
                } else {
                    Some(self.parse_pseudo_class_selector())
                }
            }
            TokenType::Delim => {
                let code = self.source().as_bytes().get(self.stream.token_start).copied().unwrap_or(0);
                match code {
                    b'.' => Some(self.parse_class_selector()),
                    b'*' | b'|' => Some(self.parse_type_selector()),
                    b'+' | b'>' | b'~' => Some(self.parse_combinator()),
                    b'&' => Some(self.parse_nesting_selector()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn selector_on_whitespace(&mut self, next: Option<&Node>, children: &mut Vec<Node>) {
        // Insert implicit descendant combinator (space) between non-combinator nodes
        if let Some(last) = children.last() {
            let last_is_combinator = matches!(last, Node::Combinator(_));
            let next_is_combinator = matches!(next, Some(Node::Combinator(_)));
            if !last_is_combinator && !next_is_combinator && next.is_some() {
                children.push(Node::Combinator(Combinator {
                    loc: None,
                    name: " ".to_string(),
                }));
            }
        }
    }

    // ── Value scope node recognizer ──

    fn value_get_node(&mut self) -> Option<Node> {
        match self.token_type() {
            TokenType::Hash => Some(self.parse_hash()),
            TokenType::Comma => Some(self.parse_operator()),
            TokenType::LeftParenthesis => Some(self.parse_parentheses()),
            TokenType::LeftSquareBracket => Some(self.parse_brackets()),
            TokenType::String => Some(self.parse_string()),
            TokenType::Dimension => Some(self.parse_dimension()),
            TokenType::Percentage => Some(self.parse_percentage()),
            TokenType::Number => Some(self.parse_number()),
            TokenType::Function => Some(self.parse_function()),
            TokenType::Url => Some(self.parse_url()),
            TokenType::Ident => Some(self.parse_identifier()),
            TokenType::Delim => {
                let code = self.source().as_bytes().get(self.stream.token_start).copied().unwrap_or(0);
                match code {
                    b'/' | b'*' | b'+' | b'-' => Some(self.parse_operator()),
                    _ => None,
                }
            }
            TokenType::Semicolon | TokenType::RightCurlyBracket | TokenType::RightParenthesis | TokenType::RightSquareBracket => None,
            _ => None,
        }
    }

    // ── Simple node parse functions ──

    fn parse_cdo(&mut self) -> Node {
        let start = self.loc_start();
        self.next();
        Node::Cdo(Cdo { loc: self.make_loc(start) })
    }

    fn parse_cdc(&mut self) -> Node {
        let start = self.loc_start();
        self.next();
        Node::Cdc(Cdc { loc: self.make_loc(start) })
    }

    fn parse_hash(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value()[1..].to_string(); // strip #
        self.next();
        Node::Hash(Hash { loc: self.make_loc(start), value })
    }

    fn parse_string(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value().to_string();
        self.next();
        Node::StringNode(StringNode { loc: self.make_loc(start), value })
    }

    fn parse_identifier(&mut self) -> Node {
        let start = self.loc_start();
        let name = self.token_value().to_string();
        self.next();
        Node::Identifier(Identifier { loc: self.make_loc(start), name })
    }

    fn parse_number(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value().to_string();
        self.next();
        Node::Number(Number { loc: self.make_loc(start), value })
    }

    fn parse_dimension(&mut self) -> Node {
        let start = self.loc_start();
        let raw = self.token_value().to_string();
        self.next();
        // Split value and unit: find where digits end
        let unit_start = raw.bytes().position(|b| {
            !b.is_ascii_digit() && b != b'.' && b != b'+' && b != b'-' && b != b'e' && b != b'E'
        }).unwrap_or(raw.len());
        let value = raw[..unit_start].to_string();
        let unit = raw[unit_start..].to_string();
        Node::Dimension(Dimension { loc: self.make_loc(start), value, unit })
    }

    fn parse_percentage(&mut self) -> Node {
        let start = self.loc_start();
        let raw = self.token_value();
        let value = raw.strip_suffix('%').unwrap_or(raw).to_string();
        self.next();
        Node::Percentage(Percentage { loc: self.make_loc(start), value })
    }

    fn parse_url(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value().to_string();
        self.next();
        Node::Url(Url { loc: self.make_loc(start), value })
    }

    fn parse_operator(&mut self) -> Node {
        let start = self.loc_start();
        let value = self.token_value().to_string();
        self.next();
        Node::Operator(Operator { loc: self.make_loc(start), value })
    }

    fn parse_function(&mut self) -> Node {
        let start = self.loc_start();
        let raw_name = self.token_value();
        let name = raw_name.strip_suffix('(').unwrap_or(raw_name).to_string();
        self.next();

        let children = self.read_sequence(
            |p| p.value_get_node(),
            |_p, _next, _children| {},
        );

        if self.token_type() == TokenType::RightParenthesis {
            self.next();
        }

        Node::Function(Function { loc: self.make_loc(start), name, children })
    }

    fn parse_parentheses(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // (

        let children = self.read_sequence(
            |p| p.value_get_node(),
            |_p, _next, _children| {},
        );

        if self.token_type() == TokenType::RightParenthesis {
            self.next();
        }

        Node::Parentheses(Parentheses { loc: self.make_loc(start), children })
    }

    fn parse_brackets(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // [

        let children = self.read_sequence(
            |p| p.value_get_node(),
            |_p, _next, _children| {},
        );

        if self.token_type() == TokenType::RightSquareBracket {
            self.next();
        }

        Node::Brackets(Brackets { loc: self.make_loc(start), children })
    }

    // ── Selector node parse functions ──

    fn parse_type_selector(&mut self) -> Node {
        let start = self.loc_start();
        let mut name = String::new();
        // Handle namespace: ns|element or *|element
        if self.token_type() == TokenType::Delim {
            let code = self.source().as_bytes()[self.stream.token_start];
            if code == b'*' || code == b'|' {
                name.push(code as char);
                self.next();
            }
        }
        if self.token_type() == TokenType::Delim && self.source().as_bytes().get(self.stream.token_start) == Some(&b'|') {
            name.push('|');
            self.next();
        }
        if self.token_type() == TokenType::Ident {
            name.push_str(self.token_value());
            self.next();
        }
        Node::TypeSelector(TypeSelector { loc: self.make_loc(start), name })
    }

    fn parse_class_selector(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // skip .
        let name = self.token_value().to_string();
        self.next();
        Node::ClassSelector(ClassSelector { loc: self.make_loc(start), name })
    }

    fn parse_id_selector(&mut self) -> Node {
        let start = self.loc_start();
        let name = self.token_value()[1..].to_string(); // strip #
        self.next();
        Node::IdSelector(IdSelector { loc: self.make_loc(start), name })
    }

    fn parse_combinator(&mut self) -> Node {
        let start = self.loc_start();
        let name = self.token_value().to_string();
        self.next();
        Node::Combinator(Combinator { loc: self.make_loc(start), name })
    }

    fn parse_nesting_selector(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // &
        Node::NestingSelector(NestingSelector { loc: self.make_loc(start) })
    }

    fn parse_attribute_selector(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // [

        self.skip_sc();
        let name_node = self.parse_identifier();

        self.skip_sc();

        let mut matcher = None;
        let mut value = None;
        let mut flags = None;

        // Check for matcher (=, ~=, |=, ^=, $=, *=)
        if self.token_type() != TokenType::RightSquareBracket && !self.stream.eof {
            let m = self.token_value().to_string();
            if m.contains('=') || self.token_type() == TokenType::Delim {
                // Read full matcher
                let mut matcher_str = String::new();
                while self.token_type() != TokenType::RightSquareBracket
                    && self.token_type() != TokenType::String
                    && self.token_type() != TokenType::Ident
                    && !self.stream.eof
                {
                    matcher_str.push_str(self.token_value());
                    self.next();
                }
                if !matcher_str.is_empty() {
                    matcher = Some(matcher_str);
                }

                self.skip_sc();

                // Read value
                if self.token_type() == TokenType::String || self.token_type() == TokenType::Ident {
                    value = Some(Box::new(if self.token_type() == TokenType::String {
                        self.parse_string()
                    } else {
                        self.parse_identifier()
                    }));
                }

                self.skip_sc();

                // Read flags (e.g. i, s)
                if self.token_type() == TokenType::Ident {
                    flags = Some(self.token_value().to_string());
                    self.next();
                }
            }
        }

        self.skip_sc();
        if self.token_type() == TokenType::RightSquareBracket {
            self.next();
        }

        Node::AttributeSelector(AttributeSelector {
            loc: self.make_loc(start),
            name: Box::new(name_node),
            matcher,
            value,
            flags,
        })
    }

    fn parse_pseudo_class_selector(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // :

        let name = self.token_value().to_string();
        let has_args = self.token_type() == TokenType::Function;

        if has_args {
            let fn_name = name.strip_suffix('(').unwrap_or(&name).to_string();
            self.next();

            let mut children = Vec::new();
            // Consume everything until matching )
            let raw_start = self.stream.token_start;
            let start_token = self.stream.token_index();
            self.stream.skip_until_balanced(start_token, |_| 0);
            let raw_value = self.source()[raw_start..self.stream.token_start].to_string();
            if !raw_value.is_empty() {
                children.push(Node::Raw(Raw { loc: None, value: raw_value }));
            }
            if self.token_type() == TokenType::RightParenthesis {
                self.next();
            }

            Node::PseudoClassSelector(PseudoClassSelector {
                loc: self.make_loc(start),
                name: fn_name,
                children: Some(children),
            })
        } else {
            self.next();
            Node::PseudoClassSelector(PseudoClassSelector {
                loc: self.make_loc(start),
                name,
                children: None,
            })
        }
    }

    fn parse_pseudo_element_selector(&mut self) -> Node {
        let start = self.loc_start();
        self.next(); // first :
        self.next(); // second :

        let name = self.token_value().to_string();
        let has_args = self.token_type() == TokenType::Function;

        if has_args {
            let fn_name = name.strip_suffix('(').unwrap_or(&name).to_string();
            self.next();

            let mut children = Vec::new();
            let raw_start = self.stream.token_start;
            let start_token = self.stream.token_index();
            self.stream.skip_until_balanced(start_token, |_| 0);
            let raw_value = self.source()[raw_start..self.stream.token_start].to_string();
            if !raw_value.is_empty() {
                children.push(Node::Raw(Raw { loc: None, value: raw_value }));
            }
            if self.token_type() == TokenType::RightParenthesis {
                self.next();
            }

            Node::PseudoElementSelector(PseudoElementSelector {
                loc: self.make_loc(start),
                name: fn_name,
                children: Some(children),
            })
        } else {
            self.next();
            Node::PseudoElementSelector(PseudoElementSelector {
                loc: self.make_loc(start),
                name,
                children: None,
            })
        }
    }
}

/// Helper trait to convert Node → Result<Node, CssSyntaxError>
trait PipeOk {
    fn pipe_ok(self) -> Result<Node, CssSyntaxError>;
}

impl PipeOk for Node {
    fn pipe_ok(self) -> Result<Node, CssSyntaxError> {
        Ok(self)
    }
}

/// Check if an at-rule name uses a style block (declarations rather than rules).
fn is_style_atrule(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "font-face" | "page" | "font-feature-values" | "viewport" | "counter-style"
    )
}

// ── Public API ──

/// Parse a CSS source string into an AST.
pub fn parse(source: &str, options: ParseOptions) -> Node {
    let mut parser = Parser::new(source, &options);

    match options.context {
        super::options::ParseContext::StyleSheet => parser.parse_stylesheet(),
        super::options::ParseContext::Selector => parser.parse_selector(),
        super::options::ParseContext::SelectorList => parser.parse_selector_list(),
        super::options::ParseContext::Value => parser.parse_value(),
        super::options::ParseContext::Declaration => {
            parser.parse_declaration().unwrap_or_else(|_| {
                Node::Raw(Raw { loc: None, value: source.to_string() })
            })
        }
        super::options::ParseContext::DeclarationList => {
            // Parse as block content without braces
            let start = parser.loc_start();
            let mut children = Vec::new();
            while !parser.stream.eof {
                match parser.token_type() {
                    TokenType::WhiteSpace | TokenType::Comment | TokenType::Semicolon => {
                        parser.next();
                    }
                    _ => {
                        let node = parser.parse_with_fallback(
                            |p| p.parse_declaration(),
                            |p| p.consume_raw(|code| if code == 0x3B { 2 } else { 0 }),
                        );
                        children.push(node);
                    }
                }
            }
            Node::DeclarationList(DeclarationList { loc: parser.make_loc(start), children })
        }
        super::options::ParseContext::Block => parser.parse_block(true),
        super::options::ParseContext::Atrule => parser.parse_atrule(),
        super::options::ParseContext::AtrulePrelude => parser.parse_atrule_prelude(),
        super::options::ParseContext::MediaQueryList => parser.parse_media_query_list(),
        super::options::ParseContext::MediaQuery => parser.parse_media_query(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_default(source: &str) -> Node {
        parse(source, ParseOptions::default())
    }

    #[test]
    fn parse_empty_stylesheet() {
        let node = parse_default("");
        assert_eq!(node.node_type(), "StyleSheet");
        if let Node::StyleSheet(ss) = &node {
            assert!(ss.children.is_empty());
        }
    }

    #[test]
    fn parse_simple_rule() {
        let node = parse_default("a { color: red }");
        assert_eq!(node.node_type(), "StyleSheet");
        if let Node::StyleSheet(ss) = &node {
            assert_eq!(ss.children.len(), 1);
            assert_eq!(ss.children[0].node_type(), "Rule");
        }
    }

    #[test]
    fn parse_declaration_value() {
        let node = parse_default("div { margin: 10px 20px }");
        if let Node::StyleSheet(ss) = &node {
            if let Node::Rule(rule) = &ss.children[0] {
                if let Node::Block(block) = rule.block.as_ref() {
                    assert_eq!(block.children.len(), 1);
                    assert_eq!(block.children[0].node_type(), "Declaration");
                }
            }
        }
    }

    #[test]
    fn parse_at_rule() {
        let node = parse_default("@media screen { div { color: red } }");
        if let Node::StyleSheet(ss) = &node {
            assert_eq!(ss.children.len(), 1);
            assert_eq!(ss.children[0].node_type(), "Atrule");
        }
    }

    #[test]
    fn parse_selector_context() {
        let node = parse(
            "div.foo > #bar",
            ParseOptions {
                context: super::super::options::ParseContext::Selector,
                ..Default::default()
            },
        );
        assert_eq!(node.node_type(), "Selector");
        if let Node::Selector(sel) = &node {
            assert!(sel.children.len() >= 3); // div.foo, >, #bar with combinators
        }
    }

    #[test]
    fn parse_value_context() {
        let node = parse(
            "10px 20px",
            ParseOptions {
                context: super::super::options::ParseContext::Value,
                ..Default::default()
            },
        );
        assert_eq!(node.node_type(), "Value");
    }
}
