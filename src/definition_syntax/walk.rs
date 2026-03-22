//! Walker for CSS Value Definition Syntax AST.

use super::types::DefinitionSyntaxNode;

/// Walk a definition syntax AST with enter/leave callbacks.
pub fn walk<E, L>(node: &DefinitionSyntaxNode, enter: &mut E, leave: &mut L)
where
    E: FnMut(&DefinitionSyntaxNode),
    L: FnMut(&DefinitionSyntaxNode),
{
    enter(node);

    match node {
        DefinitionSyntaxNode::Group(g) => {
            for term in &g.terms {
                walk(term, enter, leave);
            }
        }
        DefinitionSyntaxNode::Multiplied(m) => {
            walk(&m.term, enter, leave);
        }
        DefinitionSyntaxNode::Boolean(b) => {
            walk(&b.term, enter, leave);
        }
        // Leaf nodes — no children
        DefinitionSyntaxNode::Type(_)
        | DefinitionSyntaxNode::Property(_)
        | DefinitionSyntaxNode::Keyword(_)
        | DefinitionSyntaxNode::AtKeyword(_)
        | DefinitionSyntaxNode::Function(_)
        | DefinitionSyntaxNode::StringValue(_)
        | DefinitionSyntaxNode::Token(_)
        | DefinitionSyntaxNode::Comma => {}
    }

    leave(node);
}

/// Walk with enter callback only (convenience wrapper).
pub fn walk_enter<F>(node: &DefinitionSyntaxNode, enter: &mut F)
where
    F: FnMut(&DefinitionSyntaxNode),
{
    walk(node, enter, &mut |_| {});
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::definition_syntax::types::*;

    #[test]
    fn walk_single_keyword() {
        let node = DefinitionSyntaxNode::Keyword(KeywordNode { name: "auto".into() });
        let mut entered = Vec::new();
        walk_enter(&node, &mut |n| entered.push(n.node_type().to_string()));
        assert_eq!(entered, vec!["Keyword"]);
    }

    #[test]
    fn walk_group() {
        let node = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() }),
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "b".into() }),
            ],
            combinator: Combinator::Bar,
            explicit: false,
            disallow_empty: false,
        });
        let mut entered = Vec::new();
        walk_enter(&node, &mut |n| entered.push(n.node_type().to_string()));
        assert_eq!(entered, vec!["Group", "Keyword", "Keyword"]);
    }

    #[test]
    fn walk_multiplied() {
        let node = DefinitionSyntaxNode::Multiplied(MultiplierNode {
            term: Box::new(DefinitionSyntaxNode::Type(TypeNode { name: "length".into(), opts: None })),
            min: 1,
            max: 0,
            comma: false,
        });
        let mut entered = Vec::new();
        walk_enter(&node, &mut |n| entered.push(n.node_type().to_string()));
        assert_eq!(entered, vec!["Multiplier", "Type"]);
    }

    #[test]
    fn walk_enter_leave_order() {
        use std::cell::RefCell;

        let node = DefinitionSyntaxNode::Group(GroupNode {
            terms: vec![
                DefinitionSyntaxNode::Keyword(KeywordNode { name: "a".into() }),
            ],
            combinator: Combinator::Space,
            explicit: false,
            disallow_empty: false,
        });
        let log = RefCell::new(Vec::new());
        walk(
            &node,
            &mut |n| log.borrow_mut().push(format!("enter {}", n.node_type())),
            &mut |n| log.borrow_mut().push(format!("leave {}", n.node_type())),
        );
        assert_eq!(*log.borrow(), vec![
            "enter Group", "enter Keyword", "leave Keyword", "leave Group"
        ]);
    }
}
