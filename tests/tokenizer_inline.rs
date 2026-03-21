//! Inline test cases ported from external/csstree/lib/__tests/tokenizer.js.
//!
//! These test TokenStream navigation, block balancing, dump, skip,
//! and skipUntilBalanced using the exact same CSS inputs as the JS tests.

use csstree::tokenizer::{
    token_stream::TokenStream,
    types::TokenType,
};

/// The CSS source used by most JS tokenizer tests.
const CSS: &str = ".test\n{\n  prop: url(foo/bar.jpg) url( a\\(\\33 \\).\\ \\\"\\\'test ) calc(1 + 1) \\x \\aa ;\n}<!--<-->\\\n";

/// Expected tokens for CSS constant above.
const EXPECTED_TYPES: &[&str] = &[
    "delim-token", "ident-token", "whitespace-token",
    "{-token", "whitespace-token", "ident-token", "colon-token", "whitespace-token",
    "url-token", "whitespace-token", "url-token", "whitespace-token",
    "function-token", "number-token", "whitespace-token", "delim-token",
    "whitespace-token", "number-token", ")-token", "whitespace-token",
    "ident-token", "whitespace-token", "ident-token", "semicolon-token",
    "whitespace-token", "}-token", "CDO-token", "delim-token",
    "CDC-token", "delim-token", "whitespace-token",
];

const EXPECTED_CHUNKS: &[&str] = &[
    ".", "test", "\n",
    "{", "\n  ", "prop", ":", " ",
    "url(foo/bar.jpg)", " ", "url( a\\(\\33 \\).\\ \\\"\\\'test )", " ",
    "calc(", "1", " ", "+", " ", "1", ")", " ",
    "\\x", " ", "\\aa ", ";", "\n",
    "}", "<!--", "<", "-->", "\\", "\n",
];

// ── TokenStream basic tests ──

#[test]
fn empty_stream() {
    let stream = TokenStream::new("");
    assert!(stream.eof);
    assert_eq!(stream.token_type, TokenType::Eof);
    assert_eq!(stream.source(), "");
}

#[test]
fn dump_matches_js() {
    let stream = TokenStream::new(CSS);
    let dump = stream.dump();
    assert_eq!(dump.len(), EXPECTED_TYPES.len());
    for (i, entry) in dump.iter().enumerate() {
        assert_eq!(
            entry.token_type.as_spec_name(),
            EXPECTED_TYPES[i],
            "dump[{i}] type mismatch"
        );
        assert_eq!(entry.chunk, EXPECTED_CHUNKS[i], "dump[{i}] chunk mismatch");
    }
}

#[test]
fn next_types() {
    let mut stream = TokenStream::new(CSS);
    let mut actual = Vec::new();
    while !stream.eof {
        actual.push(stream.token_type.as_spec_name().to_string());
        stream.next();
    }
    let expected: Vec<String> = EXPECTED_TYPES.iter().map(|s| (*s).to_string()).collect();
    assert_eq!(actual, expected);
}

#[test]
fn next_start_offsets() {
    let mut stream = TokenStream::new(CSS);
    let mut actual = Vec::new();
    while !stream.eof {
        actual.push(stream.token_start);
        stream.next();
    }
    // Compute expected start offsets from chunks
    let mut expected = Vec::new();
    let mut offset = 0;
    for chunk in EXPECTED_CHUNKS {
        expected.push(offset);
        offset += chunk.len();
    }
    assert_eq!(actual, expected);
}

#[test]
fn next_end_offsets() {
    let mut stream = TokenStream::new(CSS);
    let mut actual = Vec::new();
    while !stream.eof {
        actual.push(stream.token_end);
        stream.next();
    }
    // Compute expected end offsets from chunks
    let mut expected = Vec::new();
    let mut offset = 0;
    for chunk in EXPECTED_CHUNKS {
        offset += chunk.len();
        expected.push(offset);
    }
    assert_eq!(actual, expected);
}

#[test]
fn skip_to_idents_and_delims() {
    let mut stream = TokenStream::new(CSS);

    // Indices of ident-token and delim-token in expected tokens
    let target_indices: Vec<usize> = EXPECTED_TYPES
        .iter()
        .enumerate()
        .filter(|(_, t)| **t == "ident-token" || **t == "delim-token")
        .map(|(i, _)| i)
        .collect();

    let mut actual_types = Vec::new();
    let mut prev_idx = 0;
    for (i, &target_idx) in target_indices.iter().enumerate() {
        let skip_count = if i == 0 { target_idx } else { target_idx - prev_idx };
        stream.skip(skip_count);
        actual_types.push(stream.token_type.as_spec_name().to_string());
        prev_idx = target_idx;
    }

    assert_eq!(actual_types.len(), 8); // 4 ident + 4 delim
    let expected_types: Vec<String> = target_indices
        .iter()
        .map(|&i| EXPECTED_TYPES[i].to_string())
        .collect();
    assert_eq!(actual_types, expected_types);
}

#[test]
fn skip_to_end() {
    let mut stream = TokenStream::new(CSS);
    stream.skip(EXPECTED_TYPES.len());
    assert!(stream.eof);
}

// ── getBlockTokenPairIndex tests ──

#[test]
fn block_pair_all_closed() {
    let stream = TokenStream::new("start fn({[()]}) end");
    let mut actual = Vec::new();
    stream.for_each_token(|_tt, _start, _end, index| {
        actual.push(
            stream
                .get_block_token_pair_index(index)
                .map_or(-1i32, |v| v as i32),
        );
    });
    assert_eq!(actual, vec![-1, -1, 9, 8, 7, 6, 5, 4, 3, 2, -1, -1]);
}

#[test]
fn block_pair_non_closed() {
    let stream = TokenStream::new("start fn(]}()[{)");
    let mut actual = Vec::new();
    stream.for_each_token(|_tt, _start, _end, index| {
        actual.push(
            stream
                .get_block_token_pair_index(index)
                .map_or(-1i32, |v| v as i32),
        );
    });
    assert_eq!(actual, vec![-1, -1, -1, -1, -1, 6, 5, -1, -1, -1]);
}

// ── skipUntilBalanced (Raw) tests ──

struct RawTestCase {
    source: &'static str,
    start_caret: usize,
    skip_caret: usize,
    mode: fn(u8) -> u8,
    expected: &'static str,
}

fn mode_left_curly(code: u8) -> u8 {
    if code == 0x7B { 1 } else { 0 }
}
fn mode_semicolon_included(code: u8) -> u8 {
    if code == 0x3B { 2 } else { 0 }
}
fn mode_none(_code: u8) -> u8 {
    0
}

const RAW_TESTS: &[RawTestCase] = &[
    RawTestCase { source: "? { }", start_caret: 0, skip_caret: 0, mode: mode_left_curly, expected: "? " },
    RawTestCase { source: "div { }", start_caret: 0, skip_caret: 0, mode: mode_left_curly, expected: "div " },
    RawTestCase { source: "foo(bar(1)(2)(3[{}])(4{}){}(5))", start_caret: 13, skip_caret: 13, mode: mode_left_curly, expected: "(3[{}])(4{})" },
    RawTestCase { source: "foo(bar(1) (2) (3[{}]) (4{}) {} (5))", start_caret: 15, skip_caret: 16, mode: mode_left_curly, expected: "(3[{}]) (4{}) " },
    RawTestCase { source: "func(a func(;))", start_caret: 5, skip_caret: 7, mode: mode_semicolon_included, expected: "a func(;)" },
    RawTestCase { source: "func(a func(;))", start_caret: 5, skip_caret: 12, mode: mode_semicolon_included, expected: "a func(;)" },
    RawTestCase { source: "func(a func(;); b)", start_caret: 5, skip_caret: 7, mode: mode_semicolon_included, expected: "a func(;);" },
    RawTestCase { source: "func()", start_caret: 5, skip_caret: 5, mode: mode_none, expected: "" },
    RawTestCase { source: "func([{}])", start_caret: 6, skip_caret: 7, mode: mode_none, expected: "{}" },
    RawTestCase { source: "func([{})", start_caret: 5, skip_caret: 6, mode: mode_none, expected: "[{})" },
    RawTestCase { source: "func(1, 2, 3) {}", start_caret: 0, skip_caret: 6, mode: mode_none, expected: "func(1, 2, 3) {}" },
];

#[test]
fn skip_until_balanced_raw_tests() {
    for (idx, test) in RAW_TESTS.iter().enumerate() {
        let mut stream = TokenStream::new(test.source);
        let mut start_token = stream.token_index();

        // Advance to start_caret position
        while stream.token_start < test.start_caret {
            stream.next();
            start_token = stream.token_index();
        }

        // Advance to skip_caret position
        while stream.token_start < test.skip_caret {
            stream.next();
        }

        stream.skip_until_balanced(start_token, test.mode);
        let actual = &test.source[test.start_caret..stream.token_start];
        assert_eq!(
            actual, test.expected,
            "Raw testcase#{idx}: source={:?}\n  expected={:?}\n  actual={:?}",
            test.source, test.expected, actual
        );
    }
}

// ── Dynamic buffer test ──

#[test]
fn dynamic_buffer() {
    let base_stream = TokenStream::new(CSS);
    let buffer_size = base_stream.token_count() + 10;
    let source = ".".repeat(buffer_size);
    let mut stream = TokenStream::new(&source);
    let mut count = 0;
    while !stream.eof {
        count += 1;
        stream.next();
    }
    assert_eq!(count, buffer_size);
}
