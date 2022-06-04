#[derive(Debug, Clone)]
pub struct Rule<Label> {
    pub index: usize,
    pub label: Label,
    pub rhs: Vec<Label>,
}

impl<Label> Rule<Label> {
    pub fn new(index: usize, label: Label, rhs: Vec<Label>) -> Self {
        Self { index, label, rhs }
    }
}

impl<Label> PartialEq for Rule<Label> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}
