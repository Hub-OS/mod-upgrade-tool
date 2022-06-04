// https://loup-vaillant.fr/tutorials/earley-parsing/recogniser

use super::EarleyItem;
use crate::{Ambiguity, CompletedEarleyItem, Rule, Token};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::Rc;

pub struct EarleyRecognizer<'parser, Label: Hash + Copy + Eq> {
    sets: Vec<Vec<EarleyItem<'parser, Label>>>,
    rules: &'parser [Rule<Label>],
    nullables: HashMap<Label, &'parser Rule<Label>>,
}

impl<'parser, Label: std::fmt::Debug + Hash + Copy + Eq> EarleyRecognizer<'parser, Label> {
    pub fn new(rules: &'parser [Rule<Label>]) -> Self {
        Self {
            rules,
            sets: Vec::new(),
            nullables: EarleyRecognizer::find_nullables(rules),
        }
    }

    fn find_nullables(rules: &'parser [Rule<Label>]) -> HashMap<Label, &'parser Rule<Label>> {
        // todo: cache?
        // https://github.com/jeffreykegler/kollos/blob/master/notes/misc/loup2.md
        // modified to include rule index

        let mut rules_by_rhs: HashMap<Label, Vec<&Rule<Label>>> = HashMap::new();

        let mut nullables = HashMap::new();
        let mut work_stack = Vec::new();

        for rule in rules.iter() {
            if rule.rhs.is_empty() {
                if !work_stack.contains(&rule.label) {
                    nullables.insert(rule.label, rule);
                    work_stack.push(rule.label);
                }

                continue;
            }

            for rhs in &rule.rhs {
                if let Some(list) = rules_by_rhs.get_mut(rhs) {
                    list.push(rule);
                } else {
                    rules_by_rhs.insert(*rhs, vec![rule]);
                }
            }
        }

        // find every rule using our found nullables
        // and resolve if they're nullable
        while !work_stack.is_empty() {
            let work_symbol = work_stack.pop().unwrap();

            let rules = if let Some(rules) = rules_by_rhs.get(&work_symbol) {
                rules
            } else {
                continue;
            };

            'rule_loop: for work_rule in rules {
                if nullables.contains_key(&work_rule.label) {
                    // already marked as nullable
                    continue;
                }

                // every rule on the rhs must be nullable
                for label in &work_rule.rhs {
                    if !nullables.contains_key(label) {
                        continue 'rule_loop;
                    }
                }

                // every rule on the rhs is nullable
                // so this rule is nullable
                nullables.insert(work_rule.label, work_rule);

                // add to the work stack to see if finding this nullable
                // changes the status of other rules
                work_stack.push(work_rule.label);
            }
        }

        nullables
    }

    pub fn recognize<'a>(
        mut self,
        entry: Label,
        tokens: &'parser [Token<'a, Label>],
    ) -> Vec<Vec<EarleyItem<'parser, Label>>> {
        let mut first_set = Vec::new();

        // initialization
        // iterating in reverse to avoid sorting later
        for rule in self.rules.iter().rev() {
            if rule.label == entry {
                first_set.push(EarleyItem::new(0, rule));
            }
        }

        self.sets.push(first_set);

        // primary loop
        // using indexes as the sets will grow during the loop
        let mut i = 0;

        // scanning can create new sets
        while i < self.sets.len() {
            let mut j = 0;

            // prediction + completion adds to the current set
            while j < self.sets[i].len() {
                let label = self.sets[i][j].next_label();

                match label {
                    None => self.complete(i, j),
                    Some(label) => {
                        self.predict(i, j, label);
                        self.scan(i, j, label, tokens);
                    }
                }

                j += 1
            }

            i += 1
        }

        self.sets
    }

    fn complete(&mut self, i: usize, j: usize) {
        let item = &self.sets[i][j];
        let completed_item = item.as_completed_item(i);

        let new_items: Vec<_> = self.sets[completed_item.start]
            .iter_mut()
            .filter(|old_item| old_item.next_label() == Some(completed_item.rule.label))
            .map(|old_item| {
                old_item.add_completed_item(completed_item.clone());
                old_item.advance()
            })
            .collect();

        for item in new_items {
            self.append_if_unique(i, item);
        }
    }

    fn scan<'a>(&mut self, i: usize, j: usize, label: Label, tokens: &[Token<'a, Label>]) {
        if tokens.get(i).map(|token| token.label) == Some(label) {
            let item = &self.sets[i][j];

            let new_item = item.advance();

            if self.sets.len() <= i + 1 {
                self.sets.push(vec![new_item]);
            } else {
                self.sets[i + 1].push(new_item);
            }
        }
    }

    fn predict(&mut self, i: usize, j: usize, label: Label) {
        // iterating in reverse to avoid sorting later
        for rule in self.rules.iter().rev() {
            if rule.label != label {
                continue;
            }

            self.append_if_unique(i, EarleyItem::new(i, rule));
        }

        // https://loup-vaillant.fr/tutorials/earley-parsing/empty-rules
        // "magic completion", Aycock & Horspool's nullable rule solution
        if let Some(rule) = self.nullables.get(&label) {
            let item = &mut self.sets[i][j];

            // create a new item
            let completed_item = CompletedEarleyItem {
                rule,
                ambiguity: Rc::new(RefCell::new(Ambiguity::new())),
                start: i,
                end: i,
            };

            item.add_completed_item(completed_item);

            let advanced_item = item.advance();
            self.append_if_unique(i, advanced_item);
        }
    }

    fn append_if_unique(&mut self, i: usize, item: EarleyItem<'parser, Label>) {
        let set = &mut self.sets[i];

        if !set.contains(&item) {
            set.push(item);
        }
    }
}
