# csstree-rs

A comprehensive CSS toolkit for Rust: fast parser, walker, generator, and lexer.

Rust port of [csstree](https://github.com/csstree/csstree) with full feature parity.

## Features

- **Parser** -- Recursive descent CSS parser with error recovery and tolerant mode
- **Generator** -- AST to minified CSS with automatic whitespace insertion
- **Walker** -- Visitor-pattern traversal with enter/leave callbacks, find, and find_all
- **Tokenizer** -- W3C CSS Syntax Level 3 compliant tokenizer (25 token types)
- **Lexer** -- CSS value validation against W3C specifications
- **Definition Syntax** -- W3C Value Definition Syntax parser and matcher

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
csstree = "0.1"
```

### Parse and Generate

```rust
use csstree::parser::{parse, ParseOptions};
use csstree::generator::{generate, GenerateOptions};

let css = ".foo { color: red; margin: 0 10px }";
let ast = parse(css, ParseOptions::default());
let output = generate(&ast, &GenerateOptions::default());

assert_eq!(output, ".foo{color:red;margin:0 10px}");
```

### Walk the AST

```rust
use csstree::parser::{parse, ParseOptions};
use csstree::walker::{walk, find, find_all};
use csstree::ast::Node;

let ast = parse("a { color: red } b { color: blue }", ParseOptions::default());

// Find all declarations
let decls = find_all(&ast, |node| matches!(node, Node::Declaration(_)));
assert_eq!(decls.len(), 2);

// Walk with enter callback
walk(&ast, |node| {
    if let Node::Declaration(d) = node {
        println!("{}:{}", d.property, generate(&d.value, &GenerateOptions::default()));
    }
});
```

### Tokenize

```rust
use csstree::tokenizer::{tokenize, TokenType};

tokenize("color: red", |token_type, start, end| {
    println!("{:?} {:?}", token_type, &"color: red"[start..end]);
});
// Ident "color"
// Colon ":"
// WhiteSpace " "
// Ident "red"
```

## Parser Options

```rust
use csstree::parser::{ParseOptions, options::ParseFlags};

let opts = ParseOptions {
    flags: ParseFlags {
        parse_value: true,           // Parse declaration values (vs raw)
        parse_rule_prelude: true,    // Parse selectors (vs raw)
        parse_atrule_prelude: true,  // Parse at-rule preludes (vs raw)
        parse_custom_property: false, // Parse --custom values (vs raw)
        positions: false,            // Include source locations
    },
    ..ParseOptions::default()
};
```

## Modules

| Module | Description |
|--------|-------------|
| `tokenizer` | CSS tokenizer -- byte-level token stream with balance tracking |
| `ast` | 49 AST node types as Rust enums and structs |
| `parser` | Context-driven recursive descent parser |
| `generator` | AST to CSS string with auto whitespace and source maps |
| `walker` | AST traversal: `walk`, `find`, `find_all` |
| `lexer` | Value validation against CSS grammar definitions |
| `definition_syntax` | W3C Value Definition Syntax parser |
| `utils` | Ident/string/URL encode and decode utilities |

## Test Coverage

Parser round-trip tests match the upstream JS csstree test suite:

```
Parser fixtures:  660/660 (100%)
Total unit tests: 442 passing
Test manifest:    2,957 test cases tracked
```

Tests are synced from the upstream JS csstree project and verified with:

```sh
cargo check && cargo test && cargo clippy -- -D warnings
```

## Compatibility

- **Rust**: 1.85+ (edition 2024)
- **Upstream**: Tracks [csstree](https://github.com/csstree/csstree) for feature parity

## License

MIT
