//! Token stream with indexed random access and block balancing.
//!
//! Stores tokens as packed `u32` values: `(type << 24) | end_offset`.
//! A parallel balance array maps block openers to their matching closers.

use super::tokenize;
use super::types::TokenType;
use super::utils::cmp_str;

const OFFSET_MASK: u32 = 0x00FF_FFFF;
const TYPE_SHIFT: u32 = 24;

/// A token stream providing indexed access over a tokenized CSS source.
///
/// Built by tokenizing a source string into packed `u32` arrays for
/// compact storage and fast random access. Also computes block balance
/// pairs (matching `{`/`}`, `(`/`)`, `[`/`]`).
#[derive(Debug)]
pub struct TokenStream {
    /// The original CSS source string.
    source: String,
    /// Packed `(token_type << 24) | end_offset` for each token, plus EOF sentinel.
    offset_and_type: Vec<u32>,
    /// Balance array: maps block opener index ↔ closer index.
    balance: Vec<u32>,
    /// Offset of the first character (after BOM if present).
    first_char_offset: usize,
    /// Total number of tokens (excluding EOF sentinel).
    token_count: usize,
    /// Current token index.
    token_index: usize,
    /// Current token type.
    pub token_type: TokenType,
    /// Current token start offset.
    pub token_start: usize,
    /// Current token end offset.
    pub token_end: usize,
    /// Whether we've reached EOF.
    pub eof: bool,
}

impl TokenStream {
    /// Create a new token stream by tokenizing the source.
    pub fn new(source: &str) -> Self {
        let mut stream = Self {
            source: String::new(),
            offset_and_type: Vec::new(),
            balance: Vec::new(),
            first_char_offset: 0,
            token_count: 0,
            token_index: 0,
            token_type: TokenType::Eof,
            token_start: 0,
            token_end: 0,
            eof: false,
        };
        stream.set_source(source);
        stream
    }

    /// Returns the source string.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the total number of tokens.
    pub fn token_count(&self) -> usize {
        self.token_count
    }

    /// Returns the current token index.
    pub fn token_index(&self) -> usize {
        self.token_index
    }

    /// Reset cursor to before the first token.
    pub fn reset(&mut self) {
        self.eof = false;
        self.token_index = usize::MAX; // will wrap to 0 on next()
        self.token_type = TokenType::Eof;
        self.token_start = self.first_char_offset;
        self.token_end = self.first_char_offset;
    }

    /// Tokenize a new source string, replacing the current contents.
    #[expect(
        clippy::cast_possible_truncation,
        reason = "packed u32 format limits source to 16MB — matches JS design"
    )]
    pub fn set_source(&mut self, source: &str) {
        let source = source.to_string();
        let source_length = source.len();

        let mut offset_and_type = Vec::with_capacity(source_length / 2);
        let mut balance_raw = Vec::with_capacity(source_length / 2);
        let mut first_char_offset: Option<usize> = None;
        let mut balance_close_type = TokenType::Eof;
        // Sentinel: usize::MAX means "no open block on the stack"
        let mut balance_start = usize::MAX;

        // Tokenize and collect
        tokenize(&source, |token_type, start, end| {
            let index = offset_and_type.len();

            // Pack type and end offset
            offset_and_type.push(((token_type as u32) << TYPE_SHIFT) | (end as u32));

            if first_char_offset.is_none() {
                first_char_offset = Some(start);
            }

            // Balance tracking — store current balance_start (or token_count sentinel)
            let balance_val = if balance_start == usize::MAX {
                // Will be clamped to token_count later
                u32::MAX
            } else {
                balance_start as u32
            };
            balance_raw.push(balance_val);

            if balance_start != usize::MAX && token_type == balance_close_type {
                let prev_balance_start = balance_raw[balance_start] as usize;
                balance_raw[balance_start] = index as u32;

                if prev_balance_start == u32::MAX as usize {
                    balance_start = usize::MAX;
                    balance_close_type = TokenType::Eof;
                } else {
                    balance_start = prev_balance_start;
                    let prev_type_raw = offset_and_type[prev_balance_start] >> TYPE_SHIFT;
                    balance_close_type = TokenType::from_u8(prev_type_raw as u8)
                        .and_then(TokenType::balance_pair)
                        .unwrap_or(TokenType::Eof);
                }
            } else if token_type.is_block_opener() {
                balance_start = index;
                balance_close_type = token_type.balance_pair().unwrap_or(TokenType::Eof);
            }
        });

        let token_count = offset_and_type.len();

        // EOF sentinel
        offset_and_type.push(((TokenType::Eof as u32) << TYPE_SHIFT) | (source_length as u32));
        balance_raw.push(token_count as u32);

        // Replace sentinel values with token_count
        for val in &mut balance_raw {
            if *val == u32::MAX {
                *val = token_count as u32;
            }
        }

        // Reverse-fill balance references (from openers to closers)
        for i in 0..token_count {
            let bs = balance_raw[i] as usize;
            if bs <= i {
                let be = balance_raw[bs] as usize;
                if be != i {
                    balance_raw[i] = be as u32;
                }
            } else if bs > token_count {
                balance_raw[i] = token_count as u32;
            }
        }

        self.source = source;
        self.first_char_offset = first_char_offset.unwrap_or(0);
        self.token_count = token_count;
        self.offset_and_type = offset_and_type;
        self.balance = balance_raw;

        self.reset();
        self.next();
    }

    /// Look up the token type at `current + offset`.
    pub fn lookup_type(&self, offset: usize) -> TokenType {
        let idx = self.token_index + offset;
        if idx < self.token_count {
            let raw = self.offset_and_type[idx] >> TYPE_SHIFT;
            TokenType::from_u8(raw as u8).unwrap_or(TokenType::Eof)
        } else {
            TokenType::Eof
        }
    }

    /// Look up the token type at `current + offset`, skipping whitespace and comments.
    pub fn lookup_type_non_sc(&self, mut idx: usize) -> TokenType {
        let mut offset = self.token_index;
        while offset < self.token_count {
            let raw = self.offset_and_type[offset] >> TYPE_SHIFT;
            let tt = TokenType::from_u8(raw as u8).unwrap_or(TokenType::Eof);
            if tt != TokenType::WhiteSpace && tt != TokenType::Comment {
                if idx == 0 {
                    return tt;
                }
                idx -= 1;
            }
            offset += 1;
        }
        TokenType::Eof
    }

    /// Look up the start offset of the token at `current + offset`.
    pub fn lookup_offset(&self, offset: usize) -> usize {
        let idx = self.token_index + offset;
        if idx < self.token_count && idx > 0 {
            (self.offset_and_type[idx - 1] & OFFSET_MASK) as usize
        } else if idx == 0 {
            self.first_char_offset
        } else {
            self.source.len()
        }
    }

    /// Look up a token's value case-insensitively at `current + offset`.
    pub fn lookup_value(&self, offset: usize, reference_str: &[u8]) -> bool {
        let idx = self.token_index + offset;
        if idx < self.token_count {
            let start = if idx > 0 {
                (self.offset_and_type[idx - 1] & OFFSET_MASK) as usize
            } else {
                self.first_char_offset
            };
            let end = (self.offset_and_type[idx] & OFFSET_MASK) as usize;
            cmp_str(self.source.as_bytes(), start, end, reference_str)
        } else {
            false
        }
    }

    /// Get the start offset for a token by index.
    pub fn get_token_start(&self, token_index: usize) -> usize {
        if token_index == self.token_index {
            return self.token_start;
        }
        if token_index > 0 {
            if token_index < self.token_count {
                (self.offset_and_type[token_index - 1] & OFFSET_MASK) as usize
            } else {
                (self.offset_and_type[self.token_count] & OFFSET_MASK) as usize
            }
        } else {
            self.first_char_offset
        }
    }

    /// Get the end offset for a token by index.
    pub fn get_token_end(&self, token_index: usize) -> usize {
        if token_index == self.token_index {
            return self.token_end;
        }
        let idx = token_index.clamp(0, self.token_count);
        (self.offset_and_type[idx] & OFFSET_MASK) as usize
    }

    /// Get the type of a token by index.
    pub fn get_token_type(&self, token_index: usize) -> TokenType {
        if token_index == self.token_index {
            return self.token_type;
        }
        let idx = token_index.clamp(0, self.token_count);
        let raw = self.offset_and_type[idx] >> TYPE_SHIFT;
        TokenType::from_u8(raw as u8).unwrap_or(TokenType::Eof)
    }

    /// Get the source substring from `start` to the current token start.
    pub fn substr_to_cursor(&self, start: usize) -> &str {
        &self.source[start..self.token_start]
    }

    /// Get the balance pair index for a block token.
    ///
    /// Returns `None` if the token is not a block opener/closer or is unbalanced.
    pub fn get_block_token_pair_index(&self, token_index: usize) -> Option<usize> {
        let tt = self.get_token_type(token_index);

        if tt.is_block_opener() {
            let pair_index = self.balance[token_index] as usize;
            let close_type = self.get_token_type(pair_index);
            if tt.balance_pair() == Some(close_type) {
                return Some(pair_index);
            }
        } else if tt.is_block_closer() {
            let pair_index = self.balance[token_index] as usize;
            let open_type = self.get_token_type(pair_index);
            if open_type.balance_pair() == Some(tt) {
                return Some(pair_index);
            }
        }

        None
    }

    /// Check if balance edge has been crossed.
    pub fn is_balance_edge(&self, token_index: usize) -> bool {
        (self.balance[self.token_index] as usize) < token_index
    }

    /// Check if current token is a specific delimiter.
    pub fn is_delim(&self, code: u8, offset: Option<usize>) -> bool {
        if let Some(off) = offset {
            self.lookup_type(off) == TokenType::Delim
                && self.source.as_bytes().get(self.lookup_offset(off)) == Some(&code)
        } else {
            self.token_type == TokenType::Delim
                && self.source.as_bytes().get(self.token_start) == Some(&code)
        }
    }

    /// Advance by `count` tokens.
    pub fn skip(&mut self, count: usize) {
        let next = self.token_index + count;
        if next < self.token_count {
            self.token_index = next;
            self.token_start = if next > 0 {
                (self.offset_and_type[next - 1] & OFFSET_MASK) as usize
            } else {
                self.first_char_offset
            };
            let item = self.offset_and_type[next];
            self.token_type =
                TokenType::from_u8((item >> TYPE_SHIFT) as u8).unwrap_or(TokenType::Eof);
            self.token_end = (item & OFFSET_MASK) as usize;
        } else {
            self.token_index = self.token_count;
            self.next();
        }
    }

    /// Advance to the next token.
    pub fn next(&mut self) {
        let next = self.token_index.wrapping_add(1);
        if next < self.token_count {
            self.token_index = next;
            self.token_start = self.token_end;
            let item = self.offset_and_type[next];
            self.token_type =
                TokenType::from_u8((item >> TYPE_SHIFT) as u8).unwrap_or(TokenType::Eof);
            self.token_end = (item & OFFSET_MASK) as usize;
        } else {
            self.eof = true;
            self.token_index = self.token_count;
            self.token_type = TokenType::Eof;
            self.token_start = self.source.len();
            self.token_end = self.source.len();
        }
    }

    /// Skip whitespace and comment tokens.
    pub fn skip_sc(&mut self) {
        while self.token_type == TokenType::WhiteSpace || self.token_type == TokenType::Comment {
            self.next();
        }
    }

    /// Skip until a balanced block boundary or a stop condition is met.
    ///
    /// `stop_consume` returns:
    /// - `1` to stop before consuming the current token
    /// - `2` to stop after consuming the current token
    /// - any other value to continue
    pub fn skip_until_balanced(
        &mut self,
        start_token: usize,
        stop_consume: impl Fn(u8) -> u8,
    ) {
        let mut cursor = start_token;

        while cursor < self.token_count {
            let balance_end = self.balance[cursor] as usize;

            if balance_end < start_token {
                break;
            }

            let offset = if cursor > 0 {
                (self.offset_and_type[cursor - 1] & OFFSET_MASK) as usize
            } else {
                self.first_char_offset
            };

            let code = self.source.as_bytes().get(offset).copied().unwrap_or(0);

            match stop_consume(code) {
                1 => break,
                2 => {
                    cursor += 1;
                    break;
                }
                _ => {
                    let tt_raw = self.offset_and_type[cursor] >> TYPE_SHIFT;
                    let tt =
                        TokenType::from_u8(tt_raw as u8).unwrap_or(TokenType::Eof);
                    if tt.is_block_opener() {
                        cursor = balance_end;
                    }
                }
            }

            cursor += 1;
        }

        self.skip(cursor - self.token_index);
    }

    /// Iterate over all tokens with a callback.
    pub fn for_each_token(&self, mut f: impl FnMut(TokenType, usize, usize, usize)) {
        let mut offset = self.first_char_offset;
        for i in 0..self.token_count {
            let start = offset;
            let item = self.offset_and_type[i];
            let end = (item & OFFSET_MASK) as usize;
            let tt = TokenType::from_u8((item >> TYPE_SHIFT) as u8).unwrap_or(TokenType::Eof);
            offset = end;
            f(tt, start, end, i);
        }
    }

    /// Dump all tokens for debugging.
    pub fn dump(&self) -> Vec<TokenDump> {
        let mut tokens = Vec::with_capacity(self.token_count);
        self.for_each_token(|tt, start, end, index| {
            tokens.push(TokenDump {
                idx: index,
                token_type: tt,
                chunk: self.source[start..end].to_string(),
                balance: self.balance[index] as usize,
            });
        });
        tokens
    }
}

/// Debug representation of a single token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenDump {
    /// Token index.
    pub idx: usize,
    /// Token type.
    pub token_type: TokenType,
    /// Source text for this token.
    pub chunk: String,
    /// Balance pair index.
    pub balance: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_source() {
        let stream = TokenStream::new("");
        assert!(stream.eof);
        assert_eq!(stream.token_count(), 0);
    }

    #[test]
    fn simple_tokens() {
        let stream = TokenStream::new("a {}");
        assert_eq!(stream.token_count(), 4); // ident, ws, {, }
    }

    #[test]
    fn token_navigation() {
        let mut stream = TokenStream::new("color: red");
        assert_eq!(stream.token_type, TokenType::Ident);
        assert_eq!(&stream.source()[stream.token_start..stream.token_end], "color");

        stream.next();
        assert_eq!(stream.token_type, TokenType::Colon);

        stream.next();
        assert_eq!(stream.token_type, TokenType::WhiteSpace);

        stream.next();
        assert_eq!(stream.token_type, TokenType::Ident);
        assert_eq!(&stream.source()[stream.token_start..stream.token_end], "red");

        stream.next();
        assert!(stream.eof);
    }

    #[test]
    fn skip_sc() {
        let mut stream = TokenStream::new("  /* comment */  a");
        stream.skip_sc();
        assert_eq!(stream.token_type, TokenType::Ident);
        assert_eq!(&stream.source()[stream.token_start..stream.token_end], "a");
    }

    #[test]
    fn block_balance() {
        let stream = TokenStream::new("fn({[()]})");
        // fn( should balance with )
        // Index: 0=fn( 1={ 2=[ 3=( 4=) 5=] 6=} 7=)
        let pair = stream.get_block_token_pair_index(0);
        assert!(pair.is_some());
        let close_idx = pair.unwrap();
        assert_eq!(stream.get_token_type(close_idx), TokenType::RightParenthesis);
    }

    #[test]
    fn lookup_type() {
        let stream = TokenStream::new("a b");
        assert_eq!(stream.lookup_type(0), TokenType::Ident);
        assert_eq!(stream.lookup_type(1), TokenType::WhiteSpace);
        assert_eq!(stream.lookup_type(2), TokenType::Ident);
        assert_eq!(stream.lookup_type(99), TokenType::Eof);
    }

    #[test]
    fn lookup_value() {
        let stream = TokenStream::new("Color");
        assert!(stream.lookup_value(0, b"color")); // case-insensitive
        assert!(!stream.lookup_value(0, b"red"));
    }

    #[test]
    fn is_delim() {
        let stream = TokenStream::new(".");
        assert!(stream.is_delim(b'.', None));
        assert!(!stream.is_delim(b'*', None));
    }

    #[test]
    fn dump_tokens() {
        let stream = TokenStream::new("a{}");
        let dump = stream.dump();
        assert_eq!(dump.len(), 3);
        assert_eq!(dump[0].chunk, "a");
        assert_eq!(dump[1].chunk, "{");
        assert_eq!(dump[2].chunk, "}");
    }

    #[test]
    fn for_each_token() {
        let stream = TokenStream::new("1px");
        let mut count = 0;
        stream.for_each_token(|tt, _start, _end, _idx| {
            count += 1;
            assert_eq!(tt, TokenType::Dimension);
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn skip_count() {
        let mut stream = TokenStream::new("a b c");
        assert_eq!(stream.token_type, TokenType::Ident);
        stream.skip(2); // skip past whitespace to 'b'
        assert_eq!(stream.token_type, TokenType::Ident);
        assert_eq!(&stream.source()[stream.token_start..stream.token_end], "b");
    }
}
