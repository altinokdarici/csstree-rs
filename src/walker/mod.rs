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

#[allow(clippy::wildcard_imports)] // All 49 AST types needed for exhaustive matching
use crate::ast::*;

// ── Walk action (control flow) ──

/// Control flow action returned by walk callbacks.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WalkAction {
    /// Continue walking normally.
    #[default]
    Continue,
    /// Skip visiting children of the current node.
    Skip,
    /// Stop the entire walk immediately.
    Break,
}

// ── Walk context (position tracking) ──

/// Tracks the current position in the AST during a walk.
///
/// Each field is set when entering the corresponding node type and restored
/// when leaving. This mirrors the JS walker's `context` object.
#[derive(Debug, Clone, Default)]
pub struct WalkContext<'a> {
    /// The root node passed to `walk()`.
    pub root: Option<&'a Node>,
    /// The nearest enclosing `StyleSheet` node.
    pub stylesheet: Option<&'a Node>,
    /// The nearest enclosing `Atrule` node.
    pub atrule: Option<&'a Node>,
    /// The nearest enclosing `AtrulePrelude` node.
    pub atrule_prelude: Option<&'a Node>,
    /// The nearest enclosing `Rule` node.
    pub rule: Option<&'a Node>,
    /// The nearest enclosing `Selector` node.
    pub selector: Option<&'a Node>,
    /// The nearest enclosing `Block` node.
    pub block: Option<&'a Node>,
    /// The nearest enclosing `Declaration` node.
    pub declaration: Option<&'a Node>,
    /// The nearest enclosing `Function` node.
    pub function: Option<&'a Node>,
}

/// Which node types can be targeted by the `visit` filter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum VisitFilter {
    /// Visit all node types (no filter).
    #[default]
    All,
    /// Only invoke callbacks for `Atrule` nodes.
    Atrule,
    /// Only invoke callbacks for `Rule` nodes.
    Rule,
    /// Only invoke callbacks for `Declaration` nodes.
    Declaration,
    /// Only invoke callbacks for a specific node type by name.
    NodeType(&'static str),
}

impl VisitFilter {
    /// Check if this filter matches the given node.
    pub fn matches(&self, node: &Node) -> bool {
        match self {
            Self::All => true,
            Self::Atrule => node.node_type() == "Atrule",
            Self::Rule => node.node_type() == "Rule",
            Self::Declaration => node.node_type() == "Declaration",
            Self::NodeType(name) => node.node_type() == *name,
        }
    }
}

/// Options for the `walk` function.
#[derive(Debug, Default)]
pub struct WalkOptions {
    /// Visit children in reverse order.
    pub reverse: bool,
    /// Only invoke callbacks for nodes matching this filter.
    pub visit: VisitFilter,
}

// ── Context field mapping ──

/// Which context field was set (used to save/restore).
#[derive(Clone, Copy)]
enum ContextField<'a> {
    /// No context field was set for this node type.
    None,
    /// A context field was set; stores the previous value for restoration.
    StyleSheet(Option<&'a Node>),
    Atrule(Option<&'a Node>),
    AtrulePrelude(Option<&'a Node>),
    Rule(Option<&'a Node>),
    Selector(Option<&'a Node>),
    Block(Option<&'a Node>),
    Declaration(Option<&'a Node>),
    Function(Option<&'a Node>),
}

/// Set the walk context field for a node type, returning a restore token.
fn set_context_for_node<'a>(
    ctx: &mut WalkContext<'a>,
    node: &'a Node,
) -> ContextField<'a> {
    match node {
        Node::StyleSheet(_) => {
            let prev = ctx.stylesheet;
            ctx.stylesheet = Some(node);
            ContextField::StyleSheet(prev)
        }
        Node::Atrule(_) => {
            let prev = ctx.atrule;
            ctx.atrule = Some(node);
            ContextField::Atrule(prev)
        }
        Node::AtrulePrelude(_) => {
            let prev = ctx.atrule_prelude;
            ctx.atrule_prelude = Some(node);
            ContextField::AtrulePrelude(prev)
        }
        Node::Rule(_) => {
            let prev = ctx.rule;
            ctx.rule = Some(node);
            ContextField::Rule(prev)
        }
        Node::Selector(_) => {
            let prev = ctx.selector;
            ctx.selector = Some(node);
            ContextField::Selector(prev)
        }
        Node::Block(_) => {
            let prev = ctx.block;
            ctx.block = Some(node);
            ContextField::Block(prev)
        }
        Node::Declaration(_) => {
            let prev = ctx.declaration;
            ctx.declaration = Some(node);
            ContextField::Declaration(prev)
        }
        Node::Function(_) => {
            let prev = ctx.function;
            ctx.function = Some(node);
            ContextField::Function(prev)
        }
        _ => ContextField::None,
    }
}

/// Restore the walk context field using a previously saved restore token.
fn restore_context<'a>(ctx: &mut WalkContext<'a>, field: ContextField<'a>) {
    match field {
        ContextField::None => {}
        ContextField::StyleSheet(prev) => ctx.stylesheet = prev,
        ContextField::Atrule(prev) => ctx.atrule = prev,
        ContextField::AtrulePrelude(prev) => ctx.atrule_prelude = prev,
        ContextField::Rule(prev) => ctx.rule = prev,
        ContextField::Selector(prev) => ctx.selector = prev,
        ContextField::Block(prev) => ctx.block = prev,
        ContextField::Declaration(prev) => ctx.declaration = prev,
        ContextField::Function(prev) => ctx.function = prev,
    }
}

// ── Child iteration ──

/// Collect references to all child nodes for a given node.
///
/// Returns child nodes in document order. For nodes with multiple child
/// fields (e.g. `Rule` has `prelude` + `block`), fields are returned in
/// their struct definition order.
#[allow(clippy::too_many_lines)] // Exhaustive match over 49 node variants
fn collect_children(node: &Node) -> Vec<&Node> {
    match node {
        // Nodes with `children: Vec<Node>`
        Node::AtrulePrelude(n) => n.children.iter().collect(),
        Node::Block(n) => n.children.iter().collect(),
        Node::Brackets(n) => n.children.iter().collect(),
        Node::Condition(n) => n.children.iter().collect(),
        Node::DeclarationList(n) => n.children.iter().collect(),
        Node::GeneralEnclosed(n) => n.children.iter().collect(),
        Node::LayerList(n) => n.children.iter().collect(),
        Node::MediaQueryList(n) => n.children.iter().collect(),
        Node::Parentheses(n) => n.children.iter().collect(),
        Node::Selector(n) => n.children.iter().collect(),
        Node::SelectorList(n) => n.children.iter().collect(),
        Node::StyleSheet(n) => n.children.iter().collect(),
        Node::Value(n) => n.children.iter().collect(),
        Node::Function(n) => n.children.iter().collect(),

        // Nodes with `children: Option<Vec<Node>>`
        Node::PseudoClassSelector(n) => {
            n.children.as_ref().map_or_else(Vec::new, |c| c.iter().collect())
        }
        Node::PseudoElementSelector(n) => {
            n.children.as_ref().map_or_else(Vec::new, |c| c.iter().collect())
        }

        // Nodes with Box<Node> fields
        Node::Atrule(n) => {
            let mut children = Vec::new();
            if let Some(prelude) = &n.prelude {
                children.push(prelude.as_ref());
            }
            if let Some(block) = &n.block {
                children.push(block.as_ref());
            }
            children
        }
        Node::AttributeSelector(n) => {
            let mut children = vec![n.name.as_ref()];
            if let Some(value) = &n.value {
                children.push(value.as_ref());
            }
            children
        }
        Node::Declaration(n) => {
            vec![n.value.as_ref()]
        }
        Node::Feature(n) => {
            n.value.as_ref().map_or_else(Vec::new, |v| vec![v.as_ref()])
        }
        Node::FeatureFunction(n) => {
            vec![n.value.as_ref()]
        }
        Node::FeatureRange(n) => {
            let mut children = vec![n.left.as_ref(), n.middle.as_ref()];
            if let Some(right) = &n.right {
                children.push(right.as_ref());
            }
            children
        }
        Node::MediaQuery(n) => {
            n.condition.as_ref().map_or_else(Vec::new, |c| vec![c.as_ref()])
        }
        Node::Nth(n) => {
            let mut children = vec![n.nth.as_ref()];
            if let Some(selector) = &n.selector {
                children.push(selector.as_ref());
            }
            children
        }
        Node::Ratio(n) => {
            let mut children = vec![n.left.as_ref()];
            if let Some(right) = &n.right {
                children.push(right.as_ref());
            }
            children
        }
        Node::Rule(n) => {
            vec![n.prelude.as_ref(), n.block.as_ref()]
        }
        Node::Scope(n) => {
            let mut children = Vec::new();
            if let Some(root) = &n.root {
                children.push(root.as_ref());
            }
            if let Some(limit) = &n.limit {
                children.push(limit.as_ref());
            }
            children
        }
        Node::SupportsDeclaration(n) => {
            vec![n.declaration.as_ref()]
        }

        // Leaf nodes — no children
        Node::AnPlusB(_)
        | Node::Cdc(_)
        | Node::Cdo(_)
        | Node::ClassSelector(_)
        | Node::Combinator(_)
        | Node::Comment(_)
        | Node::Dimension(_)
        | Node::Hash(_)
        | Node::Identifier(_)
        | Node::IdSelector(_)
        | Node::Layer(_)
        | Node::NestingSelector(_)
        | Node::Number(_)
        | Node::Operator(_)
        | Node::Percentage(_)
        | Node::Raw(_)
        | Node::StringNode(_)
        | Node::TypeSelector(_)
        | Node::UnicodeRange(_)
        | Node::Url(_)
        | Node::WhiteSpace(_) => Vec::new(),
    }
}

// ── Fast traversal ──

/// For Atrule/Rule/Declaration visit filters, only descend into container nodes.
///
/// This matches the JS walker's `createFastTraveralMap` which limits iteration
/// to `StyleSheet`/`Atrule`/`Rule`/`Block` (and `DeclarationList` for Declaration).
fn is_fast_traversal_container(node: &Node, filter: VisitFilter) -> bool {
    match filter {
        VisitFilter::Atrule | VisitFilter::Rule => matches!(
            node,
            Node::StyleSheet(_) | Node::Atrule(_) | Node::Rule(_) | Node::Block(_)
        ),
        VisitFilter::Declaration => matches!(
            node,
            Node::StyleSheet(_)
                | Node::Atrule(_)
                | Node::Rule(_)
                | Node::Block(_)
                | Node::DeclarationList(_)
        ),
        // No fast traversal for All or custom NodeType filters
        VisitFilter::All | VisitFilter::NodeType(_) => true,
    }
}

// ── Core walk implementation ──

/// Internal recursive walk function.
fn walk_node<'a, E, L>(
    node: &'a Node,
    ctx: &mut WalkContext<'a>,
    enter: &mut E,
    leave: &mut L,
    filter: VisitFilter,
    reverse: bool,
) -> bool
where
    E: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
    L: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
{
    // Call enter (only if filter matches or filter is All)
    let enter_action = if filter.matches(node) {
        enter(node, ctx)
    } else {
        WalkAction::Continue
    };

    if enter_action == WalkAction::Break {
        return true;
    }

    if enter_action != WalkAction::Skip && is_fast_traversal_container(node, filter) {
        // Set context for this node type
        let prev_ctx = set_context_for_node(ctx, node);

        // Visit children
        let children = collect_children(node);
        let broke = if reverse {
            children
                .iter()
                .rev()
                .any(|child| walk_node(child, ctx, enter, leave, filter, reverse))
        } else {
            children
                .iter()
                .any(|child| walk_node(child, ctx, enter, leave, filter, reverse))
        };

        // Restore context
        restore_context(ctx, prev_ctx);

        if broke {
            return true;
        }
    }

    // Call leave (only if filter matches)
    if filter.matches(node) {
        let leave_action = leave(node, ctx);
        if leave_action == WalkAction::Break {
            return true;
        }
    }

    false
}

// ── Public API ──

/// Walk an AST node depth-first, calling `enter` for each node.
///
/// The callback receives the current node and the walk context.
/// Return `WalkAction::Break` to stop the walk, `WalkAction::Skip` to skip
/// children, or `WalkAction::Continue` to keep going.
pub fn walk<'a, F>(root: &'a Node, mut enter: F)
where
    F: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
{
    let mut ctx = WalkContext { root: Some(root), ..WalkContext::default() };
    let mut noop_leave = |_: &Node, _: &WalkContext<'_>| WalkAction::Continue;
    walk_node(root, &mut ctx, &mut enter, &mut noop_leave, VisitFilter::All, false);
}

/// Walk an AST node with full options: enter/leave callbacks, reverse, visit filter.
pub fn walk_full<'a, E, L>(
    root: &'a Node,
    options: &WalkOptions,
    mut enter: E,
    mut leave: L,
)
where
    E: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
    L: FnMut(&'a Node, &WalkContext<'a>) -> WalkAction,
{
    let mut ctx = WalkContext { root: Some(root), ..WalkContext::default() };
    walk_node(root, &mut ctx, &mut enter, &mut leave, options.visit, options.reverse);
}

/// Find the first node matching a predicate (depth-first).
pub fn find<'a, F>(root: &'a Node, mut predicate: F) -> Option<&'a Node>
where
    F: FnMut(&'a Node, &WalkContext<'a>) -> bool,
{
    let mut found = None;
    walk(root, |node, ctx| {
        if predicate(node, ctx) {
            found = Some(node);
            WalkAction::Break
        } else {
            WalkAction::Continue
        }
    });
    found
}

/// Find the last node matching a predicate (reverse depth-first).
pub fn find_last<'a, F>(root: &'a Node, mut predicate: F) -> Option<&'a Node>
where
    F: FnMut(&'a Node, &WalkContext<'a>) -> bool,
{
    let mut found = None;
    let mut ctx = WalkContext { root: Some(root), ..WalkContext::default() };
    let mut enter = |node: &'a Node, ctx: &WalkContext<'a>| {
        if predicate(node, ctx) {
            found = Some(node);
            WalkAction::Break
        } else {
            WalkAction::Continue
        }
    };
    let mut noop_leave = |_: &Node, _: &WalkContext<'_>| WalkAction::Continue;
    walk_node(root, &mut ctx, &mut enter, &mut noop_leave, VisitFilter::All, true);
    found
}

/// Find all nodes matching a predicate (depth-first).
pub fn find_all<'a, F>(root: &'a Node, mut predicate: F) -> Vec<&'a Node>
where
    F: FnMut(&'a Node, &WalkContext<'a>) -> bool,
{
    let mut found = Vec::new();
    walk(root, |node, ctx| {
        if predicate(node, ctx) {
            found.push(node);
        }
        WalkAction::Continue
    });
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_action_default_is_continue() {
        assert_eq!(WalkAction::default(), WalkAction::Continue);
    }

    #[test]
    fn walk_action_equality() {
        assert_ne!(WalkAction::Break, WalkAction::Skip);
        assert_ne!(WalkAction::Skip, WalkAction::Continue);
        assert_ne!(WalkAction::Break, WalkAction::Continue);
    }

    #[test]
    fn walk_context_default_is_all_none() {
        let ctx = WalkContext::default();
        assert!(ctx.root.is_none());
        assert!(ctx.stylesheet.is_none());
        assert!(ctx.atrule.is_none());
        assert!(ctx.atrule_prelude.is_none());
        assert!(ctx.rule.is_none());
        assert!(ctx.selector.is_none());
        assert!(ctx.block.is_none());
        assert!(ctx.declaration.is_none());
        assert!(ctx.function.is_none());
    }

    #[test]
    fn walk_context_tracks_references() {
        let node = Node::Rule(Rule {
            loc: None,
            prelude: Box::new(Node::Raw(Raw { loc: None, value: "div".into() })),
            block: Box::new(Node::Block(Block { loc: None, children: vec![] })),
        });
        let mut ctx = WalkContext::default();
        ctx.root = Some(&node);
        ctx.rule = Some(&node);
        assert!(ctx.root.is_some());
        assert_eq!(ctx.root.unwrap().node_type(), "Rule");
    }

    #[test]
    fn visit_filter_matches_all() {
        let filter = VisitFilter::All;
        let node = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&node));
    }

    #[test]
    fn visit_filter_matches_specific_type() {
        let filter = VisitFilter::Declaration;
        let decl = Node::Declaration(Declaration {
            loc: None,
            important: false,
            property: "color".into(),
            value: Box::new(Node::Value(Value { loc: None, children: vec![] })),
        });
        let ident = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&decl));
        assert!(!filter.matches(&ident));
    }

    #[test]
    fn visit_filter_node_type_string() {
        let filter = VisitFilter::NodeType("Function");
        let func = Node::Function(Function { loc: None, name: "rgb".into(), children: vec![] });
        let ident = Node::Identifier(Identifier { loc: None, name: "test".into() });
        assert!(filter.matches(&func));
        assert!(!filter.matches(&ident));
    }

    #[test]
    fn walk_options_default() {
        let opts = WalkOptions::default();
        assert!(!opts.reverse);
        assert_eq!(opts.visit, VisitFilter::All);
    }

    // ── walk() tests ──

    fn make_simple_ast() -> Node {
        // .a { color: red }
        Node::StyleSheet(StyleSheet {
            loc: None,
            children: vec![Node::Rule(Rule {
                loc: None,
                prelude: Box::new(Node::SelectorList(SelectorList {
                    loc: None,
                    children: vec![Node::Selector(Selector {
                        loc: None,
                        children: vec![Node::ClassSelector(ClassSelector {
                            loc: None,
                            name: "a".into(),
                        })],
                    })],
                })),
                block: Box::new(Node::Block(Block {
                    loc: None,
                    children: vec![Node::Declaration(Declaration {
                        loc: None,
                        important: false,
                        property: "color".into(),
                        value: Box::new(Node::Value(Value {
                            loc: None,
                            children: vec![Node::Identifier(Identifier {
                                loc: None,
                                name: "red".into(),
                            })],
                        })),
                    })],
                })),
            })],
        })
    }

    #[test]
    fn walk_visits_all_nodes() {
        let ast = make_simple_ast();
        let mut types = Vec::new();
        walk(&ast, |node, _ctx| {
            types.push(node.node_type().to_string());
            WalkAction::Continue
        });
        // StyleSheet > Rule > SelectorList > Selector > ClassSelector > Block > Declaration > Value > Identifier
        assert_eq!(types, vec![
            "StyleSheet", "Rule", "SelectorList", "Selector", "ClassSelector",
            "Block", "Declaration", "Value", "Identifier"
        ]);
    }

    #[test]
    fn walk_break_stops_early() {
        let ast = make_simple_ast();
        let mut count = 0;
        walk(&ast, |node, _ctx| {
            count += 1;
            if node.node_type() == "Rule" {
                WalkAction::Break
            } else {
                WalkAction::Continue
            }
        });
        assert_eq!(count, 2); // StyleSheet, then Rule (break)
    }

    #[test]
    fn walk_skip_skips_children() {
        let ast = make_simple_ast();
        let mut types = Vec::new();
        walk(&ast, |node, _ctx| {
            types.push(node.node_type().to_string());
            if node.node_type() == "SelectorList" {
                WalkAction::Skip
            } else {
                WalkAction::Continue
            }
        });
        // SelectorList children (Selector, ClassSelector) should be skipped
        // but Block and its children should still be visited
        assert!(types.contains(&"SelectorList".to_string()));
        assert!(!types.contains(&"ClassSelector".to_string()));
        assert!(types.contains(&"Block".to_string()));
    }

    #[test]
    fn walk_context_is_set() {
        let ast = make_simple_ast();
        let mut saw_rule_context = false;
        walk(&ast, |node, ctx| {
            if node.node_type() == "Declaration" {
                // When visiting a Declaration, the Rule context should be set
                if ctx.rule.is_some() {
                    saw_rule_context = true;
                }
            }
            WalkAction::Continue
        });
        assert!(saw_rule_context);
    }

    #[test]
    fn walk_full_with_leave() {
        let ast = make_simple_ast();
        let mut enter_types = Vec::new();
        let mut leave_types = Vec::new();
        walk_full(
            &ast,
            &WalkOptions::default(),
            |node, _ctx| {
                enter_types.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |node, _ctx| {
                leave_types.push(node.node_type().to_string());
                WalkAction::Continue
            },
        );
        // Enter and leave should have same nodes but leave is in post-order
        assert_eq!(enter_types.len(), leave_types.len());
        assert_eq!(enter_types[0], "StyleSheet"); // first entered
        assert_eq!(leave_types[0], "ClassSelector"); // first to leave (deepest leaf in first branch)
    }

    #[test]
    fn walk_full_reverse() {
        let ast = make_simple_ast();
        let mut types = Vec::new();
        walk_full(
            &ast,
            &WalkOptions { reverse: true, visit: VisitFilter::All },
            |node, _ctx| {
                types.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |_, _| WalkAction::Continue,
        );
        // In reverse, Block comes before SelectorList
        let block_pos = types.iter().position(|t| t == "Block").unwrap();
        let selector_list_pos = types.iter().position(|t| t == "SelectorList").unwrap();
        assert!(block_pos < selector_list_pos);
    }

    #[test]
    fn walk_full_visit_filter() {
        let ast = make_simple_ast();
        let mut visited = Vec::new();
        walk_full(
            &ast,
            &WalkOptions { reverse: false, visit: VisitFilter::Declaration },
            |node, _ctx| {
                visited.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |_, _| WalkAction::Continue,
        );
        // Only Declaration should be in the visited list
        assert_eq!(visited, vec!["Declaration"]);
    }

    // ── find/find_all tests ──

    #[test]
    fn find_returns_first_match() {
        let ast = make_simple_ast();
        let result = find(&ast, |node, _ctx| node.node_type() == "ClassSelector");
        assert!(result.is_some());
        assert_eq!(result.unwrap().node_type(), "ClassSelector");
    }

    #[test]
    fn find_returns_none_when_no_match() {
        let ast = make_simple_ast();
        let result = find(&ast, |node, _ctx| node.node_type() == "Dimension");
        assert!(result.is_none());
    }

    #[test]
    fn find_last_returns_last_match() {
        // Build AST with two identifiers
        let ast = Node::Value(Value {
            loc: None,
            children: vec![
                Node::Identifier(Identifier { loc: None, name: "first".into() }),
                Node::Identifier(Identifier { loc: None, name: "second".into() }),
            ],
        });
        let result = find_last(&ast, |node, _ctx| node.node_type() == "Identifier");
        assert!(result.is_some());
        // In reverse walk, "second" is visited first
        if let Node::Identifier(id) = result.unwrap() {
            assert_eq!(id.name, "second");
        } else {
            panic!("Expected Identifier node");
        }
    }

    #[test]
    fn find_all_collects_matches() {
        let ast = make_simple_ast();
        let results = find_all(&ast, |node, _ctx| {
            matches!(node, Node::Identifier(_) | Node::ClassSelector(_))
        });
        assert_eq!(results.len(), 2); // ClassSelector + Identifier
    }

    #[test]
    fn walk_leaf_node() {
        let leaf = Node::Number(Number { loc: None, value: "42".into() });
        let mut count = 0;
        walk(&leaf, |_node, _ctx| {
            count += 1;
            WalkAction::Continue
        });
        assert_eq!(count, 1);
    }

    #[test]
    fn walk_function_node_children() {
        let func = Node::Function(Function {
            loc: None,
            name: "rgb".into(),
            children: vec![
                Node::Number(Number { loc: None, value: "255".into() }),
                Node::Operator(Operator { loc: None, value: ",".into() }),
                Node::Number(Number { loc: None, value: "0".into() }),
            ],
        });
        let mut types = Vec::new();
        walk(&func, |node, _ctx| {
            types.push(node.node_type().to_string());
            WalkAction::Continue
        });
        assert_eq!(types, vec!["Function", "Number", "Operator", "Number"]);
    }

    #[test]
    fn walk_function_context_is_set() {
        let func = Node::Function(Function {
            loc: None,
            name: "rgb".into(),
            children: vec![
                Node::Number(Number { loc: None, value: "255".into() }),
            ],
        });
        let mut fn_context_set = false;
        walk(&func, |node, ctx| {
            if node.node_type() == "Number" && ctx.function.is_some() {
                fn_context_set = true;
            }
            WalkAction::Continue
        });
        assert!(fn_context_set);
    }

    // ── Fast traversal tests ──

    #[test]
    fn fast_traversal_declaration_filter_skips_non_container() {
        let ast = make_simple_ast();
        let mut all_types = Vec::new();
        walk_full(
            &ast,
            &WalkOptions { reverse: false, visit: VisitFilter::Declaration },
            |node, _ctx| {
                all_types.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |_, _| WalkAction::Continue,
        );
        // With fast traversal, only Declaration callback fires
        // It should NOT visit SelectorList, Selector, ClassSelector, Value, Identifier
        assert_eq!(all_types, vec!["Declaration"]);
    }

    #[test]
    fn fast_traversal_rule_filter() {
        let ast = make_simple_ast();
        let mut visited = Vec::new();
        walk_full(
            &ast,
            &WalkOptions { reverse: false, visit: VisitFilter::Rule },
            |node, _ctx| {
                visited.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |_, _| WalkAction::Continue,
        );
        assert_eq!(visited, vec!["Rule"]);
    }

    #[test]
    fn fast_traversal_atrule_filter() {
        // Build: @media screen { .a { color: red } }
        let ast = Node::StyleSheet(StyleSheet {
            loc: None,
            children: vec![Node::Atrule(Atrule {
                loc: None,
                name: "media".into(),
                prelude: Some(Box::new(Node::AtrulePrelude(AtrulePrelude {
                    loc: None,
                    children: vec![Node::Identifier(Identifier {
                        loc: None,
                        name: "screen".into(),
                    })],
                }))),
                block: Some(Box::new(Node::Block(Block {
                    loc: None,
                    children: vec![Node::Rule(Rule {
                        loc: None,
                        prelude: Box::new(Node::SelectorList(SelectorList {
                            loc: None,
                            children: vec![],
                        })),
                        block: Box::new(Node::Block(Block {
                            loc: None,
                            children: vec![],
                        })),
                    })],
                }))),
            })],
        });
        let mut visited = Vec::new();
        walk_full(
            &ast,
            &WalkOptions { reverse: false, visit: VisitFilter::Atrule },
            |node, _ctx| {
                visited.push(node.node_type().to_string());
                WalkAction::Continue
            },
            |_, _| WalkAction::Continue,
        );
        assert_eq!(visited, vec!["Atrule"]);
    }

    #[test]
    fn walk_atrule_children() {
        // Verify walker properly visits Atrule child fields
        let ast = Node::Atrule(Atrule {
            loc: None,
            name: "import".into(),
            prelude: Some(Box::new(Node::AtrulePrelude(AtrulePrelude {
                loc: None,
                children: vec![Node::StringNode(StringNode {
                    loc: None,
                    value: "foo.css".into(),
                })],
            }))),
            block: None,
        });
        let mut types = Vec::new();
        walk(&ast, |node, _ctx| {
            types.push(node.node_type().to_string());
            WalkAction::Continue
        });
        assert_eq!(types, vec!["Atrule", "AtrulePrelude", "String"]);
    }

    #[test]
    fn leave_break_stops_walk() {
        let ast = make_simple_ast();
        let mut leave_count = 0;
        walk_full(
            &ast,
            &WalkOptions::default(),
            |_, _| WalkAction::Continue,
            |_node, _ctx| {
                leave_count += 1;
                WalkAction::Break // break on first leave
            },
        );
        assert_eq!(leave_count, 1);
    }
}
