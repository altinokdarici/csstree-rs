//! CSS identifier encoding and decoding.
//!
//! Handles CSS escape sequences in identifiers.

/// Decode a CSS identifier, resolving escape sequences.
///
/// E.g., `r#"\61 bc"` → `"abc"` (where `\61` is the hex escape for `a`).
pub fn decode_ident(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let bytes = value.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\\' && i + 1 < bytes.len() {
            i += 1;
            // Try hex escape: \HHHHHH
            let hex_start = i;
            let mut hex_end = i;
            while hex_end < bytes.len()
                && hex_end - hex_start < 6
                && bytes[hex_end].is_ascii_hexdigit()
            {
                hex_end += 1;
            }

            if hex_end > hex_start {
                let hex = &value[hex_start..hex_end];
                if let Ok(code) = u32::from_str_radix(hex, 16) {
                    if let Some(ch) = char::from_u32(code) {
                        result.push(ch);
                    } else {
                        result.push('\u{FFFD}');
                    }
                }
                // Skip optional whitespace after hex escape
                if hex_end < bytes.len() && bytes[hex_end].is_ascii_whitespace() {
                    hex_end += 1;
                }
                i = hex_end;
            } else {
                // Non-hex escape: just include the character
                result.push(bytes[i] as char);
                i += 1;
            }
        } else {
            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Encode a string as a CSS identifier, adding escape sequences where needed.
pub fn encode_ident(value: &str) -> String {
    let mut result = String::with_capacity(value.len());

    for (i, ch) in value.chars().enumerate() {
        if ch == '-' && i == 0 && value.len() == 1 {
            result.push_str("\\-");
        } else if i == 0 && ch.is_ascii_digit() {
            // Leading digit must be escaped
            use std::fmt::Write;
            result.push('\\');
            let _ = write!(result, "{:X} ", ch as u32);
        } else if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch > '\x7f' {
            result.push(ch);
        } else if ch == '\0' {
            result.push('\u{FFFD}');
        } else {
            result.push('\\');
            result.push(ch);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_simple() {
        assert_eq!(decode_ident("color"), "color");
    }

    #[test]
    fn decode_hex_escape() {
        assert_eq!(decode_ident("\\61 bc"), "abc");
    }

    #[test]
    fn decode_non_hex_escape() {
        assert_eq!(decode_ident("\\!important"), "!important");
    }

    #[test]
    fn encode_simple() {
        assert_eq!(encode_ident("color"), "color");
    }

    #[test]
    fn encode_special_chars() {
        let encoded = encode_ident("my.class");
        assert!(encoded.contains("\\"));
    }

    #[test]
    fn encode_leading_digit() {
        let encoded = encode_ident("0abc");
        assert_eq!(encoded, "\\30 abc");
    }

    #[test]
    fn encode_hyphen_only() {
        assert_eq!(encode_ident("-"), "\\-");
    }

    #[test]
    fn round_trip() {
        let original = "my-class";
        assert_eq!(decode_ident(&encode_ident(original)), original);
    }
}
