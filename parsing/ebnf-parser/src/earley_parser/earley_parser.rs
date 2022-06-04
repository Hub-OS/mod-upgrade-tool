// https://loup-vaillant.fr/tutorials/earley-parsing/recogniser

use super::EarleyRecognizer;
use crate::{ASTNode, ParserError, Rule, Token};
use std::hash::Hash;

#[derive(Default)]
pub struct EarleyParser<Label: Copy + Eq + Hash> {
    entry: Label,
    rules: Vec<Rule<Label>>,
    hidden_rules: Vec<Label>,
}

impl<Label: std::fmt::Debug + Copy + Eq + Hash> EarleyParser<Label> {
    pub fn new(entry: Label) -> Self {
        Self {
            entry,
            rules: Vec::new(),
            hidden_rules: Vec::new(),
        }
    }

    pub fn add_rule<I>(&mut self, label: Label, rhs: I)
    where
        I: std::iter::IntoIterator<Item = Label>,
    {
        self.rules.push(Rule::new(
            self.rules.len(),
            label,
            rhs.into_iter().collect(),
        ));
    }

    pub fn add_rules<L, R>(&mut self, label: Label, rules: L)
    where
        L: std::iter::IntoIterator<Item = R>,
        R: std::iter::IntoIterator<Item = Label>,
    {
        let start_index = self.rules.len();
        self.rules.extend(
            rules
                .into_iter()
                .enumerate()
                .map(|(i, rhs)| Rule::new(start_index + i, label, rhs.into_iter().collect())),
        );
    }

    pub fn hide_rule(&mut self, label: Label) {
        self.hidden_rules.push(label);
    }

    pub fn parse<'a>(
        &self,
        source: &'a str,
        tokens: &[Token<'a, Label>],
    ) -> Result<ASTNode<'a, Label>, ParserError<'a, Label>> {
        let recognizer = EarleyRecognizer::new(&self.rules);
        let sets = recognizer.recognize(self.entry, tokens);

        // handle UnexpectedToken
        if sets.len() - 1 < tokens.len() {
            let token = tokens[sets.len() - 1];

            let (line, col) = crate::get_line_and_col(source, token.offset);
            return Err(ParserError::UnexpectedToken { token, line, col });
        }

        // find the root_item
        let root_item =
            sets.last().unwrap().iter().rev().find(|item| {
                item.start == 0 && item.is_complete() && item.rule.label == self.entry
            });

        if let Some(root_item) = root_item {
            Ok(root_item
                .as_completed_item(sets.len() - 1)
                .as_node(tokens, &self.hidden_rules))
        } else {
            // root_item did not complete, parsing expected more tokens
            Err(ParserError::UnexpectedEOF)
        }
    }
}
