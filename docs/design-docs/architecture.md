# Architecture

## Overview

csstree-rs is a Rust reimplementation of `@eslint/css-tree`, a comprehensive CSS toolkit providing: tokenizer, parser, AST, generator, walker, lexer (validation), and definition syntax parser.

## Module Map

```
src/
├── tokenizer/          # W3C CSS Syntax Level 3 tokenizer
│   ├── types.rs        # 25 token type enum
│   ├── char_code_definitions.rs  # Character classification
│   ├── names.rs        # Token type → string name mapping
│   ├── utils.rs        # Consume functions (ident, number, string, url, etc.)
│   ├── token_stream.rs # TokenStream: indexed access over token sequence
│   └── offset_to_location.rs    # byte offset → (line, column) mapping
│
├── ast/                # 49 AST node types as Rust enums/structs
│   └── mod.rs          # Node enum, Location, all node structs
│
├── parser/             # Recursive descent parser
│   └── mod.rs          # parse() entry, context-driven parsing, error recovery
│
├── generator/          # AST → CSS string
│   └── mod.rs          # generate() entry, whitespace insertion, source maps
│
├── walker/             # AST traversal
│   └── mod.rs          # walk(), find(), find_all(), enter/leave callbacks
│
├── definition_syntax/  # W3C Value Definition Syntax parser
│   └── mod.rs          # parse/generate/walk for definition syntax
│
├── lexer/              # CSS value validation against specs
│   └── mod.rs          # Lexer, match graphs, generic types, An+B, URange
│
└── utils/              # Shared utilities
    └── mod.rs          # ident/string/url encode/decode, clone, List
```

## Dependency Direction (enforced at compile time)

```
tokenizer ←─── parser
     ↑              ↓
     │            ast ──→ generator
     │              ↓
     │           walker
     │
definition_syntax ←─── lexer (also depends on tokenizer + ast)

utils (leaf — used by any module)
```

No circular dependencies. Each module declares its deps explicitly via `use crate::`.

## JS → Rust Module Mapping

| JS Path | Rust Module | Notes |
|---------|-------------|-------|
| `lib/tokenizer/` | `src/tokenizer/` | Direct port, zero-copy on source |
| `lib/syntax/node/` | `src/ast/` | 49 nodes → Rust enum + structs |
| `lib/parser/` | `src/parser/` | Recursive descent, same parse contexts |
| `lib/generator/` | `src/generator/` | Same whitespace logic |
| `lib/walker/` | `src/walker/` | Visitor pattern with enter/leave |
| `lib/definition-syntax/` | `src/definition_syntax/` | Standalone |
| `lib/lexer/` | `src/lexer/` | Match graphs, generic types |
| `lib/utils/` | `src/utils/` | List → Vec or custom collection |

## Key Design Decisions

### AST Representation
- JS uses a custom doubly-linked `List` for children. Rust uses `Vec<Node>` (cache-friendly, simpler ownership).
- Each node is a struct. A top-level `Node` enum wraps all 49 types for type-safe dispatch.
- Location info is `Option<Location>` — only present when parsing with `positions: true`.

### Tokenizer
- Operates on `&[u8]` (byte slice) of the source for speed.
- Token stores `(token_type, start, end)` — value is a slice into source, not a copy.
- Token types are a `#[repr(u8)]` enum for compact storage and fast matching.

### Error Handling
- Parse errors → `Result<Node, CssSyntaxError>` or error recovery into `Raw` nodes.
- Lexer validation errors → structured `MatchError` type.
- No panics in the public API.

### Performance Targets
- Tokenizer: competitive with or faster than `cssparser` (Servo's tokenizer).
- Parser: single-pass, no backtracking except for documented ambiguities.
- Zero-copy: AST nodes borrow from source string via lifetime `'a`.
