use crate::Rule;
use std::collections::HashMap;
use std::hash::Hash;

pub fn find_nullables<Label: Hash + Copy + Eq>(
    rules: &[Rule<Label>],
) -> HashMap<Label, &Rule<Label>> {
    // https://github.com/jeffreykegler/kollos/blob/master/notes/misc/loup2.md
    // modified to include the rule as a value for generating a completed item

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
