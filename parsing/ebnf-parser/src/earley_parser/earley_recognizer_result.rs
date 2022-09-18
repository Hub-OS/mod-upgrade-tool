use super::{Ambiguity, EarleyItem};
use crate::{ASTNode, ASTNodeLabel, EarleyParser, Rule, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct EarleyRecognizerResult<'parser, Label: ASTNodeLabel> {
    sets: Vec<Vec<EarleyItem<'parser, Label>>>,
    ambiguities: Vec<Rc<RefCell<Ambiguity<'parser, Label>>>>,
    nullables: &'parser HashMap<Label, &'parser Rule<Label>>,
}

impl<'parser, Label: ASTNodeLabel> EarleyRecognizerResult<'parser, Label> {
    pub fn new(
        sets: Vec<Vec<EarleyItem<'parser, Label>>>,
        ambiguities: Vec<Rc<RefCell<Ambiguity<'parser, Label>>>>,
        nullables: &'parser HashMap<Label, &'parser Rule<Label>>,
    ) -> Self {
        Self {
            sets,
            ambiguities,
            nullables,
        }
    }

    #[cfg(test)]
    pub fn sets(&self) -> &[Vec<EarleyItem<'parser, Label>>] {
        &self.sets
    }

    pub fn len(&self) -> usize {
        self.sets.len()
    }

    pub fn into_ast<'a>(
        mut self,
        parser: &'parser EarleyParser<Label>,
        tokens: &[Token<'a, Label>],
    ) -> Option<ASTNode<'a, Label>> {
        let last_set = self.sets.last_mut().unwrap();

        // sort the last set for operator precedence
        last_set.sort_by_key(|item| item.rule.index);

        // find the root_item
        let root_item = last_set.iter().find(|item| {
            item.start == 0 && item.is_complete() && item.rule.label == parser.entry()
        })?;

        // create ast before destroying ambiguity
        let node = root_item.as_completed_item(tokens.len()).as_node(
            parser.hidden_rules(),
            self.nullables,
            tokens,
        );

        // destroy circular references
        for ambiguity in self.ambiguities {
            ambiguity.borrow_mut().clear_completed_items();
        }

        Some(node)
    }
}
