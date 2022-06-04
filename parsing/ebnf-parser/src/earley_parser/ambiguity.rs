use crate::{CompletedEarleyItem, Rule};

#[derive(Debug)]
pub struct Ambiguity<'parser, Label: Copy> {
    /// the different interpretations of the same branch
    interpretations: Vec<Vec<CompletedEarleyItem<'parser, Label>>>,
}

impl<'parser, Label: Copy> Ambiguity<'parser, Label> {
    pub fn new() -> Self {
        Self {
            interpretations: Vec::new(),
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
    }

    pub fn disambiguate(
        &mut self,
        rule: &Rule<Label>,
        start: usize,
        end: usize,
    ) -> Vec<Option<CompletedEarleyItem<'parser, Label>>> {
        while self.interpretations.len() < rule.rhs.len() {
            self.interpretations.push(Vec::new());
        }

        // we iterated items in the recognizer in reverse for this step
        // items that are completed later (and therefore a longer match) are added to the end of the list
        // iterating in reverse makes the rules created first appear lower in the list
        // we want higher priority to items closer to the top of the rule list and higher priority to longer matches
        // so we just need to search for them in reverse instead of requiring a sort here

        struct WorkItem {
            start: usize,
            reversed_index: usize,
        }

        let mut work_list = vec![WorkItem {
            start,
            reversed_index: 0,
        }];

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
                        // correct the reversed index
                        let symbol_index =
                            symbol_interpretations.len() - work_item.reversed_index - 1;
                        output.push(Some(symbol_interpretations[symbol_index].clone()));
                    }
                }

                return output;
            }

            let is_last_item = rhs_index == self.interpretations.len() - 1;
            let symbol_interpretations = &self.interpretations[rhs_index];

            if symbol_interpretations.is_empty() && work_item.reversed_index == 0 {
                // empty list = this is a token
                // if this is the first attempt, add a work item, otherwise fallthrough into the next check
                let start = work_item.start + 1;

                if is_last_item && start != end {
                    // mismatched end, bubble up failure
                    work_list.pop();
                    work_list.last_mut().unwrap().reversed_index += 1;
                    continue;
                }

                work_list.push(WorkItem {
                    start,
                    reversed_index: 0,
                });
                continue;
            }

            if work_item.reversed_index >= symbol_interpretations.len() {
                // failed every interpretation
                // try the next interpretation in the previous work item
                work_list.pop();
                work_list.last_mut().unwrap().reversed_index += 1;
                continue;
            }

            // need to correct the reversed index
            let symbol_index = symbol_interpretations.len() - work_item.reversed_index - 1;
            let completed_item = &symbol_interpretations[symbol_index];

            if work_item.start != completed_item.start
                || (is_last_item && completed_item.end != end)
            {
                // item does not follow the previous rule
                // or it doesn't end where we expect it to
                // try the next interpretation
                work_list.last_mut().unwrap().reversed_index += 1;
                continue;
            }

            let start = completed_item.end;
            work_list.push(WorkItem {
                start,
                reversed_index: 0,
            });
        }
    }
}
