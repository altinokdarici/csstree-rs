# Phase 2: AST Types

**Status:** NOT STARTED
**Depends on:** Nothing
**Unlocks:** Parser (Phase 3), Generator (Phase 4), Walker (Phase 5)

## Goal

Define all 49 AST node types from `external/csstree/lib/syntax/node/` as Rust types.

## JS Source

All 49 files in `external/csstree/lib/syntax/node/` — each defines a node's `name`, `structure`, `parse()`, and `generate()`.

## Node Types (49)

StyleSheet, Rule, Selector, SelectorList, Declaration, DeclarationList, Block, Value, Function, Identifier, IdSelector, ClassSelector, TypeSelector, AttributeSelector, PseudoClassSelector, PseudoElementSelector, Combinator, NestingSelector, Number, Dimension, Percentage, Hash, String, Url, Brackets, Parentheses, Operator, Ratio, UnicodeRange, Atrule, AtrulePrelude, MediaQuery, MediaQueryList, Condition, Feature, FeatureFunction, FeatureRange, Nth, AnPlusB, Comment, CDO, CDC, Raw, Layer, LayerList, Scope, GeneralEnclosed, SupportsDeclaration, WhiteSpace

## Steps

1. Read each JS node file to understand its `structure` field
2. Define a `Location` struct (source, start offset/line/col, end offset/line/col)
3. Define each node as a struct with typed fields
4. Define a `Node` enum wrapping all 49 types
5. Derive `Debug, Clone, PartialEq` on all types
6. Add `#[cfg(test)]` tests verifying struct construction
7. Verify `cargo check` passes

## Acceptance Criteria

- [ ] All 49 node types defined as structs
- [ ] `Node` enum covers all types
- [ ] All types derive Debug, Clone, PartialEq
- [ ] Location tracking support
- [ ] Children stored as `Vec<Node>` (not linked list)
