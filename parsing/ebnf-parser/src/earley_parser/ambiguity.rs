use crate::{CompletedEarleyItem, Rule};
use std::collections::HashMap;
use std::hash::Hash;

pub struct Ambiguity<'parser, Label: Copy> {
    /// the different interpretations of the same branch
    interpretations: Vec<Vec<CompletedEarleyItem<'parser, Label>>>,
    sorted: bool,
}

impl<'parser, Label: Copy + Hash + Eq> Ambiguity<'parser, Label> {
    pub fn new() -> Self {
        Self {
            interpretations: Vec::new(),
            sorted: true,
        }
    }

    pub fn add_completed_item(
        &mut self,
        completed_item: CompletedEarleyItem<'parser, Label>,
        rhs_index: usize,
    ) {
        while self.interpretations.len() <= rhs_index {
            self.interpretations.push(Vec::new());
        }

        self.interpretations[rhs_index].push(completed_item);
        self.sorted = false;
    }

    pub fn clear_completed_items(&mut self) {
        self.interpretations.clear();
    }

    pub fn resolve(
        &mut self,
        nullables: &HashMap<Label, &'parser Rule<Label>>,
        visited_items: &[CompletedEarleyItem<'parser, Label>],
        rule: &Rule<Label>,
        start: usize,
        end: usize,
    ) -> Vec<Option<CompletedEarleyItem<'parser, Label>>> {
        while self.interpretations.len() < rule.rhs.len() {
            self.interpretations.push(Vec::new());
        }

        if !self.sorted {
            for index_interpretations in &mut self.interpretations {
                index_interpretations
                    .sort_by_key(|interpretation| (interpretation.rule.index, interpretation.end));
            }
            self.sorted = true;
        }

        struct WorkItem {
            start: usize,
            index: usize,
        }

        let mut work_list = vec![WorkItem { start, index: 0 }];

        loop {
            let work_item = work_list.last().unwrap();
            let rhs_index = work_list.len() - 1;

            if rhs_index >= self.interpretations.len() {
                // we're done!

                // last item is extra
                work_list.pop();

                // create the output
                let mut output = Vec::new();

                for (rhs_index, work_item) in work_list.into_iter().enumerate() {
                    let symbol_interpretations = &self.interpretations[rhs_index];

                    if symbol_interpretations.is_empty() {
                        // token
                        output.push(None);
                    } else {
                        output.push(Some(symbol_interpretations[work_item.index].clone()));
                    }
                }

                return output;
            }

            let is_last_item = rhs_index == self.interpretations.len() - 1;
            let symbol_interpretations = &self.interpretations[rhs_index];

            if symbol_interpretations.is_empty() && work_item.index == 0 {
                // empty list = this is either a token or a nullable rule
                // only attempt if this is the first try, otherwise fallthrough into the fail check

                let start = if nullables.contains_key(&rule.rhs[rhs_index]) {
                    // nullable rule
                    work_item.start
                } else {
                    // this is a token
                    work_item.start + 1
                };

                if is_last_item && start != end {
                    // mismatched end, bubble up failure
                    work_list.pop();
                    work_list.last_mut().unwrap().index += 1;
                    continue;
                }

                work_list.push(WorkItem { start, index: 0 });
                continue;
            }

            if work_item.index >= symbol_interpretations.len() {
                // failed every interpretation
                // try the next interpretation in the previous work item
                work_list.pop();
                work_list.last_mut().unwrap().index += 1;
                continue;
            }

            let completed_item = &symbol_interpretations[work_item.index];

            let acceptable_start = work_item.start == completed_item.start;
            let acceptable_end = !is_last_item || completed_item.end == end;
            let acceptable_item = !visited_items.contains(&completed_item);

            if !acceptable_start || !acceptable_end || !acceptable_item {
                // try the next interpretation
                work_list.last_mut().unwrap().index += 1;
                continue;
            }

            let start = completed_item.end;
            work_list.push(WorkItem { start, index: 0 });
        }
    }
}
