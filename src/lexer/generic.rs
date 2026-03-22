//! Built-in generic type matchers for CSS value validation.
//!
//! Each matcher takes a token slice and returns how many tokens were consumed
//! (0 = no match).

use crate::tokenizer::types::TokenType;

/// A generic type matcher function signature.
///
/// Takes a token type and value, returns number of tokens consumed (0 = no match).
pub type GenericMatcher = fn(token_type: TokenType, value: &str) -> usize;

/// Get a built-in generic matcher by name.
pub fn get_generic_matcher(name: &str) -> Option<GenericMatcher> {
    match name {
        "ident" => Some(match_ident),
        "custom-ident" => Some(match_custom_ident),
        "dashed-ident" => Some(match_dashed_ident),
        "string" => Some(match_string),
        "number" => Some(match_number),
        "integer" => Some(match_integer),
        "zero" => Some(match_zero),
        "percentage" => Some(match_percentage),
        "hex-color" => Some(match_hex_color),
        "length" => Some(match_length),
        "angle" => Some(match_angle),
        "time" => Some(match_time),
        "frequency" => Some(match_frequency),
        "resolution" => Some(match_resolution),
        "flex" => Some(match_flex),
        "dimension" => Some(match_dimension),
        "ident-token" => Some(match_ident_token),
        "function-token" => Some(match_function_token),
        "number-token" => Some(match_number_token),
        "string-token" => Some(match_string_token),
        "hash-token" => Some(match_hash_token),
        "percentage-token" => Some(match_percentage_token),
        "dimension-token" => Some(match_dimension_token),
        _ => None,
    }
}

// ── Token type matchers ──

fn match_ident(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Ident)
}

fn match_custom_ident(tt: TokenType, val: &str) -> usize {
    if tt != TokenType::Ident {
        return 0;
    }
    let lower = val.to_ascii_lowercase();
    usize::from(!matches!(
        lower.as_str(),
        "initial" | "inherit" | "unset" | "revert" | "revert-layer" | "default"
    ))
}

fn match_dashed_ident(tt: TokenType, val: &str) -> usize {
    usize::from(tt == TokenType::Ident && val.starts_with("--"))
}

fn match_string(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::String)
}

fn match_number(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Number)
}

fn match_integer(tt: TokenType, val: &str) -> usize {
    usize::from(tt == TokenType::Number && !val.contains('.'))
}

fn match_zero(tt: TokenType, val: &str) -> usize {
    usize::from(tt == TokenType::Number && val == "0")
}

fn match_percentage(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Percentage)
}

fn match_hex_color(tt: TokenType, val: &str) -> usize {
    if tt != TokenType::Hash {
        return 0;
    }
    let hex = val.strip_prefix('#').unwrap_or(val);
    let len = hex.len();
    usize::from(
        matches!(len, 3 | 4 | 6 | 8) && hex.chars().all(|c| c.is_ascii_hexdigit()),
    )
}

// ── Dimension matchers ──

fn match_dimension(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Dimension)
}

fn match_length(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("length").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    // 0 is a valid length
    if tt == TokenType::Number && val == "0" {
        return 1;
    }
    0
}

fn match_angle(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("angle").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    if tt == TokenType::Number && val == "0" {
        return 1;
    }
    0
}

fn match_time(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("time").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    0
}

fn match_frequency(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("frequency").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    0
}

fn match_resolution(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("resolution").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    0
}

fn match_flex(tt: TokenType, val: &str) -> usize {
    if tt == TokenType::Dimension {
        let unit = extract_unit(val).to_ascii_lowercase();
        if super::units::UNITS.get("flex").is_some_and(|u| u.contains(&unit.as_str())) {
            return 1;
        }
    }
    0
}

/// Extract the unit portion from a dimension value (e.g., "10px" → "px").
fn extract_unit(val: &str) -> &str {
    let bytes = val.as_bytes();
    let mut i = 0;
    // Skip number part (digits, dot, sign)
    while i < bytes.len() {
        let ch = bytes[i];
        if ch.is_ascii_digit() || ch == b'.' || ch == b'-' || ch == b'+' {
            i += 1;
        } else {
            break;
        }
    }
    &val[i..]
}

// ── Raw token type matchers ──

fn match_ident_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Ident)
}

fn match_function_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Function)
}

fn match_number_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Number)
}

fn match_string_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::String)
}

fn match_hash_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Hash)
}

fn match_percentage_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Percentage)
}

fn match_dimension_token(tt: TokenType, _val: &str) -> usize {
    usize::from(tt == TokenType::Dimension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ident_matcher() {
        assert_eq!(match_ident(TokenType::Ident, "auto"), 1);
        assert_eq!(match_ident(TokenType::Number, "42"), 0);
    }

    #[test]
    fn custom_ident_excludes_keywords() {
        assert_eq!(match_custom_ident(TokenType::Ident, "foo"), 1);
        assert_eq!(match_custom_ident(TokenType::Ident, "initial"), 0);
        assert_eq!(match_custom_ident(TokenType::Ident, "inherit"), 0);
        assert_eq!(match_custom_ident(TokenType::Ident, "default"), 0);
    }

    #[test]
    fn dashed_ident_matcher() {
        assert_eq!(match_dashed_ident(TokenType::Ident, "--my-var"), 1);
        assert_eq!(match_dashed_ident(TokenType::Ident, "my-var"), 0);
    }

    #[test]
    fn hex_color_matcher() {
        assert_eq!(match_hex_color(TokenType::Hash, "#fff"), 1);
        assert_eq!(match_hex_color(TokenType::Hash, "#ff0000"), 1);
        assert_eq!(match_hex_color(TokenType::Hash, "#ff000080"), 1);
        assert_eq!(match_hex_color(TokenType::Hash, "#gg0000"), 0);
        assert_eq!(match_hex_color(TokenType::Hash, "#f"), 0);
    }

    #[test]
    fn length_matcher() {
        assert_eq!(match_length(TokenType::Dimension, "10px"), 1);
        assert_eq!(match_length(TokenType::Dimension, "2em"), 1);
        assert_eq!(match_length(TokenType::Number, "0"), 1);
        assert_eq!(match_length(TokenType::Number, "5"), 0);
    }

    #[test]
    fn integer_matcher() {
        assert_eq!(match_integer(TokenType::Number, "42"), 1);
        assert_eq!(match_integer(TokenType::Number, "3.14"), 0);
    }

    #[test]
    fn extract_unit_works() {
        assert_eq!(extract_unit("10px"), "px");
        assert_eq!(extract_unit("2.5em"), "em");
        assert_eq!(extract_unit("0"), "");
    }

    #[test]
    fn generic_matcher_lookup() {
        assert!(get_generic_matcher("length").is_some());
        assert!(get_generic_matcher("color").is_none());
        assert!(get_generic_matcher("ident").is_some());
    }
}
