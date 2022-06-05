// I couldn't understand Elizabeth Scott's solution to converting an Earley recognizer into a parser
// And loup's depth first search appeared to be way too slow for many grammars so this is a custom solution
// Did not work out the big O for this

use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

use crate::{ASTNode, Ambiguity, Rule, Token};

struct AsNodeWorkItem<'parser, 'a, Label: Copy> {
    children: Vec<ASTNode<'a, Label>>,
    rule: &'parser Rule<Label>,
    items: Vec<Option<CompletedEarleyItem<'parser, Label>>>,
    start: usize,
    symbol_index: usize,
}

impl<'parser, 'a, Label: std::fmt::Debug + Copy + Hash + Eq> AsNodeWorkItem<'parser, 'a, Label> {
    fn new(
        nullables: &HashMap<Label, &'parser Rule<Label>>,
        item: &CompletedEarleyItem<'parser, Label>,
    ) -> Self {
        Self {
            children: Vec::new(),
            rule: item.rule,
            items: item
                .ambiguity
                .borrow_mut()
                .resolve(nullables, item.rule, item.start, item.end),
            start: item.start,
            symbol_index: 0,
        }
    }

    fn current_item_start(&self) -> usize {
        let mut start = self.start;

        if self.symbol_index == 0 {
            start
        } else {
            let mut token_count = 0;

            // find the last rule, use the end of the rule as the start
            for i in (0..self.symbol_index).rev() {
                if let Some(completed_item) = &self.items[i] {
                    start = completed_item.end;
                    break;
                } else {
                    token_count += 1;
                }
            }

            start + token_count
        }
    }

    fn into_node(self) -> ASTNode<'a, Label> {
        ASTNode::Branch {
            label: self.rule.label,
            children: self.children,
        }
    }
}

#[derive(Clone)]
pub struct CompletedEarleyItem<'parser, Label: Copy> {
    pub rule: &'parser Rule<Label>,
    pub ambiguity: Rc<RefCell<Ambiguity<'parser, Label>>>,
    pub start: usize,
    pub end: usize,
}

impl<'parser, Label: std::fmt::Debug + Copy + Hash + Eq> CompletedEarleyItem<'parser, Label> {
    pub fn as_node<'a>(
        &self,
        hidden_rules: &[Label],
        nullables: &HashMap<Label, &'parser Rule<Label>>,
        tokens: &[Token<'a, Label>],
    ) -> ASTNode<'a, Label> {
        let mut work_items = vec![AsNodeWorkItem::new(nullables, self)];

        loop {
            let work_item = work_items.last_mut().unwrap();
            let symbol_index = work_item.symbol_index;

            if symbol_index >= work_item.items.len() {
                // completed the work item
                let node = work_items.pop().unwrap().into_node();

                if let Some(work_item) = work_items.last_mut() {
                    // append children to the last work item
                    if hidden_rules.contains(&node.label()) {
                        // hidden rules have their children added to the parent and not the node
                        work_item.children.extend(node.into_children().unwrap());
                    } else {
                        work_item.children.push(node);
                    }

                    work_item.symbol_index += 1;
                    continue;
                } else {
                    // completed all work items, return this node as the result
                    return node;
                }
            }

            if let Some(completed_item) = &work_item.items[symbol_index] {
                // rule
                let new_work_item = AsNodeWorkItem::new(nullables, completed_item);
                work_items.push(new_work_item);
            } else {
                // token
                work_item
                    .children
                    .push(ASTNode::new_leaf(tokens[work_item.current_item_start()]));

                work_item.symbol_index += 1;
            }
        }
    }
}
