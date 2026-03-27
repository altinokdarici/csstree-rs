//! Token preparation for CSS value matching.
//!
//! Converts CSS value strings into token arrays for the matching algorithm.

use crate::tokenizer::{tokenize, types::TokenType};

/// A prepared token for matching.
#[derive(Debug, Clone, PartialEq)]
pub struct PreparedToken {
    /// Token type.
    pub token_type: TokenType,
    /// Token value as string.
    pub value: String,
}

/// Tokenize a CSS value string into prepared tokens.
pub fn prepare_tokens(value: &str) -> Vec<PreparedToken> {
    let mut result = Vec::new();

    tokenize(value, |tt, start, end| {
        let val = &value[start..end];
        result.push(PreparedToken {
            token_type: tt,
            value: val.to_string(),
        });
    });

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple_value() {
        let tokens = prepare_tokens("red");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Ident);
        assert_eq!(tokens[0].value, "red");
    }

    #[test]
    fn tokenize_dimension() {
        let tokens = prepare_tokens("10px");
        assert_eq!(tokens.len(), 1);
        assert_eq!(tokens[0].token_type, TokenType::Dimension);
        assert_eq!(tokens[0].value, "10px");
    }

    #[test]
    fn tokenize_multiple_values() {
        let tokens = prepare_tokens("1px solid red");
        assert!(tokens.len() >= 3);
    }

    #[test]
    fn tokenize_hash_color() {
        let tokens = prepare_tokens("#ff0000");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].token_type, TokenType::Hash);
    }

}
