//! CSS token types per W3C CSS Syntax Module Level 3.
//!
//! Each variant maps to a CSS token type. The discriminant values match
//! the JS `types.js` constants exactly for fixture compatibility.

/// CSS token type as defined by the CSS Syntax Module Level 3 spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TokenType {
    /// End of file.
    Eof = 0,
    /// `<ident-token>` — an identifier like `color` or `--custom`.
    Ident = 1,
    /// `<function-token>` — an identifier followed by `(`, like `rgb(`.
    Function = 2,
    /// `<at-keyword-token>` — `@` followed by an identifier, like `@media`.
    AtKeyword = 3,
    /// `<hash-token>` — `#` followed by name characters, like `#fff`.
    Hash = 4,
    /// `<string-token>` — a quoted string like `"hello"` or `'world'`.
    String = 5,
    /// `<bad-string-token>` — a string with an unescaped newline.
    BadString = 6,
    /// `<url-token>` — an unquoted URL like `url(foo.png)`.
    Url = 7,
    /// `<bad-url-token>` — a malformed URL token.
    BadUrl = 8,
    /// `<delim-token>` — a single code point not matched by anything else.
    Delim = 9,
    /// `<number-token>` — a numeric value like `42` or `3.14`.
    Number = 10,
    /// `<percentage-token>` — a number followed by `%`.
    Percentage = 11,
    /// `<dimension-token>` — a number followed by a unit like `10px`.
    Dimension = 12,
    /// `<whitespace-token>` — one or more whitespace characters.
    WhiteSpace = 13,
    /// `<CDO-token>` — `<!--`.
    Cdo = 14,
    /// `<CDC-token>` — `-->`.
    Cdc = 15,
    /// `<colon-token>` — `:`.
    Colon = 16,
    /// `<semicolon-token>` — `;`.
    Semicolon = 17,
    /// `<comma-token>` — `,`.
    Comma = 18,
    /// `<[-token>` — `[`.
    LeftSquareBracket = 19,
    /// `<]-token>` — `]`.
    RightSquareBracket = 20,
    /// `<(-token>` — `(`.
    LeftParenthesis = 21,
    /// `<)-token>` — `)`.
    RightParenthesis = 22,
    /// `<{-token>` — `{`.
    LeftCurlyBracket = 23,
    /// `<}-token>` — `}`.
    RightCurlyBracket = 24,
    /// CSS comment `/* ... */`.
    Comment = 25,
}

impl TokenType {
    /// Total number of token types (including EOF).
    pub const COUNT: usize = 26;

    /// Convert a `u8` discriminant to a `TokenType`.
    ///
    /// Returns `None` if the value is out of range.
    pub fn from_u8(value: u8) -> Option<Self> {
        (value < 26).then(||
            // SAFETY: all values 0..26 are valid discriminants of this #[repr(u8)] enum.
            unsafe { std::mem::transmute::<u8, Self>(value) }
        )
    }

    /// Returns the CSS spec name for this token type (e.g. `"ident-token"`).
    pub fn as_spec_name(self) -> &'static str {
        TOKEN_NAMES[self as usize]
    }

    /// Returns `true` if this token opens a block (`Function`, `(`, `[`, `{`).
    pub fn is_block_opener(self) -> bool {
        matches!(
            self,
            Self::Function
                | Self::LeftParenthesis
                | Self::LeftSquareBracket
                | Self::LeftCurlyBracket
        )
    }

    /// Returns `true` if this token closes a block (`)`, `]`, `}`).
    pub fn is_block_closer(self) -> bool {
        matches!(
            self,
            Self::RightParenthesis | Self::RightSquareBracket | Self::RightCurlyBracket
        )
    }

    /// Returns the matching closing token type for a block opener.
    ///
    /// Returns `None` if this token is not a block opener.
    pub fn balance_pair(self) -> Option<Self> {
        match self {
            Self::Function | Self::LeftParenthesis => Some(Self::RightParenthesis),
            Self::LeftSquareBracket => Some(Self::RightSquareBracket),
            Self::LeftCurlyBracket => Some(Self::RightCurlyBracket),
            _ => None,
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_spec_name())
    }
}

/// CSS spec names indexed by token type discriminant.
const TOKEN_NAMES: [&str; TokenType::COUNT] = [
    "EOF-token",
    "ident-token",
    "function-token",
    "at-keyword-token",
    "hash-token",
    "string-token",
    "bad-string-token",
    "url-token",
    "bad-url-token",
    "delim-token",
    "number-token",
    "percentage-token",
    "dimension-token",
    "whitespace-token",
    "CDO-token",
    "CDC-token",
    "colon-token",
    "semicolon-token",
    "comma-token",
    "[-token",
    "]-token",
    "(-token",
    ")-token",
    "{-token",
    "}-token",
    "comment-token",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_type_discriminants_match_js() {
        assert_eq!(TokenType::Eof as u8, 0);
        assert_eq!(TokenType::Ident as u8, 1);
        assert_eq!(TokenType::Function as u8, 2);
        assert_eq!(TokenType::AtKeyword as u8, 3);
        assert_eq!(TokenType::Hash as u8, 4);
        assert_eq!(TokenType::String as u8, 5);
        assert_eq!(TokenType::BadString as u8, 6);
        assert_eq!(TokenType::Url as u8, 7);
        assert_eq!(TokenType::BadUrl as u8, 8);
        assert_eq!(TokenType::Delim as u8, 9);
        assert_eq!(TokenType::Number as u8, 10);
        assert_eq!(TokenType::Percentage as u8, 11);
        assert_eq!(TokenType::Dimension as u8, 12);
        assert_eq!(TokenType::WhiteSpace as u8, 13);
        assert_eq!(TokenType::Cdo as u8, 14);
        assert_eq!(TokenType::Cdc as u8, 15);
        assert_eq!(TokenType::Colon as u8, 16);
        assert_eq!(TokenType::Semicolon as u8, 17);
        assert_eq!(TokenType::Comma as u8, 18);
        assert_eq!(TokenType::LeftSquareBracket as u8, 19);
        assert_eq!(TokenType::RightSquareBracket as u8, 20);
        assert_eq!(TokenType::LeftParenthesis as u8, 21);
        assert_eq!(TokenType::RightParenthesis as u8, 22);
        assert_eq!(TokenType::LeftCurlyBracket as u8, 23);
        assert_eq!(TokenType::RightCurlyBracket as u8, 24);
        assert_eq!(TokenType::Comment as u8, 25);
    }

    #[test]
    fn token_type_count() {
        assert_eq!(TokenType::COUNT, 26);
    }

    #[test]
    fn from_u8_valid() {
        for i in 0..TokenType::COUNT as u8 {
            assert!(TokenType::from_u8(i).is_some());
        }
    }

    #[test]
    fn from_u8_invalid() {
        assert!(TokenType::from_u8(26).is_none());
        assert!(TokenType::from_u8(255).is_none());
    }

    #[test]
    fn spec_names_match_js() {
        assert_eq!(TokenType::Eof.as_spec_name(), "EOF-token");
        assert_eq!(TokenType::Ident.as_spec_name(), "ident-token");
        assert_eq!(TokenType::Function.as_spec_name(), "function-token");
        assert_eq!(TokenType::Comment.as_spec_name(), "comment-token");
        assert_eq!(TokenType::LeftCurlyBracket.as_spec_name(), "{-token");
    }

    #[test]
    fn block_opener_closer() {
        assert!(TokenType::Function.is_block_opener());
        assert!(TokenType::LeftParenthesis.is_block_opener());
        assert!(TokenType::LeftSquareBracket.is_block_opener());
        assert!(TokenType::LeftCurlyBracket.is_block_opener());
        assert!(!TokenType::Ident.is_block_opener());

        assert!(TokenType::RightParenthesis.is_block_closer());
        assert!(TokenType::RightSquareBracket.is_block_closer());
        assert!(TokenType::RightCurlyBracket.is_block_closer());
        assert!(!TokenType::Ident.is_block_closer());
    }

    #[test]
    fn balance_pairs() {
        assert_eq!(TokenType::Function.balance_pair(), Some(TokenType::RightParenthesis));
        assert_eq!(TokenType::LeftParenthesis.balance_pair(), Some(TokenType::RightParenthesis));
        assert_eq!(TokenType::LeftSquareBracket.balance_pair(), Some(TokenType::RightSquareBracket));
        assert_eq!(TokenType::LeftCurlyBracket.balance_pair(), Some(TokenType::RightCurlyBracket));
        assert_eq!(TokenType::Ident.balance_pair(), None);
    }
}
