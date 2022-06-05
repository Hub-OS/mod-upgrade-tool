use crate::Token;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ASTNode<'a, Label: Clone> {
    Leaf {
        token: Token<'a, Label>,
    },
    Branch {
        label: Label,
        children: Vec<ASTNode<'a, Label>>,
    },
}

impl<'a, Label: Copy> ASTNode<'a, Label> {
    pub fn new_branch(label: Label) -> Self {
        Self::Branch {
            label,
            children: Vec::new(),
        }
    }

    pub fn new_leaf(token: Token<'a, Label>) -> Self {
        Self::Leaf { token }
    }

    pub fn label(&self) -> Label {
        match self {
            Self::Leaf { token } => token.label,
            Self::Branch { label, .. } => *label,
        }
    }

    pub fn token(&self) -> Option<&Token<'a, Label>> {
        match self {
            Self::Leaf { token } => Some(token),
            Self::Branch { .. } => None,
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<ASTNode<'a, Label>>> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { children, .. } => Some(children),
        }
    }

    pub fn children(&self) -> Option<&Vec<ASTNode<'a, Label>>> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { children, .. } => Some(children),
        }
    }

    pub fn into_children(self) -> Option<Vec<ASTNode<'a, Label>>> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { children, .. } => Some(children),
        }
    }

    pub fn start(&self) -> usize {
        match self {
            Self::Leaf { token } => token.offset,
            Self::Branch { children, .. } => children.first().map(|node| node.start()).unwrap_or(0),
        }
    }

    pub fn end(&self) -> usize {
        match self {
            Self::Leaf { token } => token.offset + token.content.len(),
            Self::Branch { children, .. } => children.last().map(|node| node.end()).unwrap_or(0),
        }
    }

    pub fn walk<F>(&self, mut callback: F)
    where
        F: FnMut(&Self, &[usize]),
    {
        let mut path = Vec::new();

        callback(self, &path);

        let mut nodes = Vec::new();

        if let Some(children) = self.children() {
            nodes.push((children, 0));
            path.push(0);
        } else {
            return;
        };

        while let Some((children, index)) = nodes.last_mut() {
            let node = match children.get(*index) {
                Some(node) => node,
                None => {
                    nodes.pop();
                    path.pop();
                    if let Some(path_index) = path.last_mut() {
                        *path_index += 1;
                    }
                    continue;
                }
            };

            callback(node, &path);

            *index += 1;

            if let Some(node_children) = node.children() {
                nodes.push((node_children, 0));
                path.push(0);
            } else {
                *path.last_mut().unwrap() += 1;
            }
        }
    }

    pub fn walk_mut<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut Self, &[usize]),
    {
        callback(self, &[]);

        let mut path = vec![0];

        while !path.is_empty() {
            let node = match self.node_at_mut(&path) {
                Some(node) => node,
                None => {
                    path.pop();
                    if let Some(index) = path.last_mut() {
                        *index += 1;
                    }
                    continue;
                }
            };

            callback(node, &path);

            if node.children().is_some() {
                path.push(0);
            } else {
                *path.last_mut().unwrap() += 1;
            }
        }
    }

    pub fn node_at(&self, path: &[usize]) -> Option<&Self> {
        let mut node = self;

        for index in path {
            node = node.children()?.get(*index)?
        }

        Some(node)
    }

    pub fn node_at_mut(&mut self, path: &[usize]) -> Option<&mut Self> {
        let mut node = self;

        for index in path {
            node = node.children_mut()?.get_mut(*index)?
        }

        Some(node)
    }
}
