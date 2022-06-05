use crate::{Ambiguity, CompletedEarleyItem, Rule};
use std::cell::RefCell;
use std::hash::Hash;
use std::rc::Rc;

#[derive(Clone)]
pub struct EarleyItem<'parser, Label: Copy> {
    pub rule: &'parser Rule<Label>,
    pub next: usize,
    pub start: usize,
    ambiguity: Rc<RefCell<Ambiguity<'parser, Label>>>,
}

impl<'parser, Label: Copy + Hash + Eq> EarleyItem<'parser, Label> {
    pub fn new(
        start: usize,
        rule: &'parser Rule<Label>,
        ambiguity: Rc<RefCell<Ambiguity<'parser, Label>>>,
    ) -> Self {
        Self {
            rule,
            next: 0,
            start,
            ambiguity,
        }
    }

    pub fn add_completed_item(&mut self, completed_item: CompletedEarleyItem<'parser, Label>) {
        self.ambiguity
            .borrow_mut()
            .add_completed_item(completed_item, self.next)
    }

    pub fn advance(&self) -> Self {
        let mut item = self.clone();
        item.next += 1;
        item
    }

    pub fn is_complete(&self) -> bool {
        self.next == self.rule.rhs.len()
    }

    pub fn next_label(&self) -> Option<Label> {
        self.rule.rhs.get(self.next).cloned()
    }

    pub fn as_completed_item(&self, end: usize) -> CompletedEarleyItem<'parser, Label> {
        CompletedEarleyItem {
            rule: self.rule,
            ambiguity: self.ambiguity.clone(),
            start: self.start,
            end,
        }
    }
}

impl<'parser, Label: Copy> PartialEq for EarleyItem<'parser, Label> {
    fn eq(&self, other: &Self) -> bool {
        self.rule.index == other.rule.index && self.start == other.start && self.next == other.next
    }
}

impl<'parser, Label: std::fmt::Debug + Copy> std::fmt::Debug for EarleyItem<'parser, Label> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}\t->", self.rule.label)?;

        for (index, label) in self.rule.rhs.iter().enumerate() {
            if self.next == index {
                write!(f, " •")?;
            }

            write!(f, " {:?}", label)?;
        }

        if self.next == self.rule.rhs.len() {
            write!(f, " •")?;
        }

        write!(f, "\t({})", self.start)?;

        Ok(())
    }
}
