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

impl<Label: Copy + Eq + Hash> EarleyParser<Label> {
    pub fn new(entry: Label) -> Self {
        Self {
            entry,
            rules: Vec::new(),
            hidden_rules: Vec::new(),
        }
    }

    pub fn entry(&self) -> Label {
        self.entry
    }

    pub fn hidden_rules(&self) -> &[Label] {
        &self.hidden_rules
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
        let nullables = super::find_nullables(&self.rules);
        let recognizer = EarleyRecognizer::new(&nullables, &self.rules);
        let result = recognizer.recognize(self.entry, tokens);

        // handle UnexpectedToken
        if result.len() - 1 < tokens.len() {
            let token = tokens[result.len() - 1];

            let (line, col) = crate::get_line_and_col(source, token.offset);
            return Err(ParserError::UnexpectedToken { token, line, col });
        }

        if let Some(node) = result.into_ast(self, tokens) {
            Ok(node)
        } else {
            // root_item did not complete, parsing expected more tokens
            Err(ParserError::UnexpectedEOF)
        }
    }
}
