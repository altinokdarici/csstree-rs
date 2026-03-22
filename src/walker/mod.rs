//! AST walker — traverses CSS AST nodes with enter/leave callbacks.
//!
//! ## Architecture (from JS `external/csstree/lib/walker/`)
//!
//! The walker provides depth-first traversal of AST nodes with callbacks.
//!
//! - **`create.js`** — Factory that creates a walker from config. The walker
//!   uses per-node-type iterators to visit child fields. Key features:
//!   - `enter` callback — called before visiting children
//!   - `leave` callback — called after visiting children
//!   - `visit` option — filter to only visit a specific node type
//!   - `reverse` option — visit children in reverse order
//!   - `break` sentinel — early termination from the walk
//!   - `skip` sentinel — skip visiting children of the current node
//!   - `context` object — tracks current position in the tree
//!     (stylesheet, atrule, rule, selector, block, declaration, function)
//!
//! - **Helper methods**:
//!   - `find(ast, fn)` — find first node matching predicate
//!   - `findLast(ast, fn)` — find last node matching predicate (reverse walk)
//!   - `findAll(ast, fn)` — find all nodes matching predicate
//!
//! ## Rust design
//!
//! - `walk(node, callback)` — simple depth-first walk with enter callback
//! - `WalkAction` enum for Break/Skip/Continue control flow
//! - `find`, `find_all` as standalone functions
//! - Each Node variant's children are known at compile time from the AST types
