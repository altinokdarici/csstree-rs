//! Integration tests that load tokenizer fixture JSON files and verify
//! that the Rust tokenizer produces matching results.

use csstree::tokenizer::{tokenize, types::TokenType};
use serde::Deserialize;
use std::fs;

#[derive(Deserialize)]
struct TokenizeFixture {
    #[serde(default)]
    valid: Vec<serde_json::Value>,
    #[serde(default)]
    invalid: Vec<serde_json::Value>,
}

#[derive(Deserialize)]
struct ExpectedToken {
    #[serde(rename = "type")]
    token_type: String,
    chunk: String,
}

/// Map a spec name (e.g. "ident-token") to a TokenType.
fn token_type_from_spec_name(name: &str) -> TokenType {
    for i in 0..TokenType::COUNT as u8 {
        if let Some(tt) = TokenType::from_u8(i) {
            if tt.as_spec_name() == name {
                return tt;
            }
        }
    }
    panic!("Unknown token type spec name: {name}");
}

/// Tokenize a string and collect all (type, chunk) pairs.
fn collect_tokens(source: &str) -> Vec<(TokenType, String)> {
    let mut tokens = Vec::new();
    tokenize(source, |tt, start, end| {
        tokens.push((tt, source[start..end].to_string()));
    });
    tokens
}

/// Test a valid fixture entry.
///
/// If it's a plain string: expect a single token of the fixture's type with that chunk.
/// If it's an object with `tokens`: expect the exact token sequence.
fn test_valid_entry(fixture_name: &str, index: usize, entry: &serde_json::Value) {
    match entry {
        serde_json::Value::String(input) => {
            let expected_type = token_type_from_spec_name(fixture_name);
            let tokens = collect_tokens(input);
            assert!(
                tokens.len() == 1 && tokens[0].0 == expected_type && tokens[0].1 == *input,
                "VALID {fixture_name}[{index}]: input={input:?}\n  \
                 expected: single {expected_type} token\n  \
                 got: {tokens:?}"
            );
        }
        serde_json::Value::Object(obj) => {
            let input = obj.get("value").and_then(|v| v.as_str()).unwrap();
            let expected_tokens: Vec<ExpectedToken> =
                serde_json::from_value(obj.get("tokens").cloned().unwrap()).unwrap();
            let actual = collect_tokens(input);

            assert_eq!(
                actual.len(),
                expected_tokens.len(),
                "VALID {fixture_name}[{index}]: input={input:?}\n  \
                 token count mismatch: expected {}, got {}",
                expected_tokens.len(),
                actual.len()
            );

            for (i, (actual_tt, actual_chunk)) in actual.iter().enumerate() {
                let expected = &expected_tokens[i];
                let expected_tt = token_type_from_spec_name(&expected.token_type);
                assert_eq!(
                    *actual_tt, expected_tt,
                    "VALID {fixture_name}[{index}] token[{i}]: type mismatch\n  \
                     input={input:?}\n  expected: {expected_tt}, got: {actual_tt}"
                );
                assert_eq!(
                    *actual_chunk, expected.chunk,
                    "VALID {fixture_name}[{index}] token[{i}]: chunk mismatch\n  \
                     input={input:?}\n  expected: {:?}, got: {actual_chunk:?}",
                    expected.chunk
                );
            }
        }
        _ => panic!("Unexpected fixture entry type"),
    }
}

/// Test an invalid fixture entry — should NOT tokenize as a single token of the fixture's type.
fn test_invalid_entry(fixture_name: &str, index: usize, entry: &serde_json::Value) {
    let expected_type = token_type_from_spec_name(fixture_name);
    let input = match entry {
        serde_json::Value::String(s) => s.as_str(),
        serde_json::Value::Object(obj) => obj.get("value").and_then(|v| v.as_str()).unwrap(),
        _ => panic!("Unexpected fixture entry type"),
    };

    let tokens = collect_tokens(input);
    let is_single_correct =
        tokens.len() == 1 && tokens[0].0 == expected_type && tokens[0].1 == input;

    assert!(
        !is_single_correct,
        "INVALID {fixture_name}[{index}]: input={input:?}\n  \
         expected: NOT a single {expected_type} token\n  \
         got: {tokens:?} (should have been rejected)"
    );
}

fn load_and_test(filename: &str) {
    let fixture_name = filename.strip_suffix(".json").unwrap();
    let path = format!("tests/fixtures/tokenize/{filename}");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {path}: {e}"));
    let fixture: TokenizeFixture =
        serde_json::from_str(&content).unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

    for (i, entry) in fixture.valid.iter().enumerate() {
        test_valid_entry(fixture_name, i, entry);
    }
    for (i, entry) in fixture.invalid.iter().enumerate() {
        test_invalid_entry(fixture_name, i, entry);
    }
}

#[test]
fn fixture_ident_token() {
    load_and_test("ident-token.json");
}

#[test]
fn fixture_function_token() {
    load_and_test("function-token.json");
}

#[test]
fn fixture_at_keyword_token() {
    load_and_test("at-keyword-token.json");
}

#[test]
fn fixture_hash_token() {
    load_and_test("hash-token.json");
}

#[test]
fn fixture_string_token() {
    load_and_test("string-token.json");
}

#[test]
fn fixture_url_token() {
    load_and_test("url-token.json");
}

#[test]
fn fixture_dimension_token() {
    load_and_test("dimension-token.json");
}
