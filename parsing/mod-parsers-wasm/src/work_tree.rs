use js_sys::Object as JsObject;

#[derive(Debug)]
pub enum WorkTreeNode<'a> {
    Leaf {
        object: JsObject,
        content: &'a str,
    },
    Branch {
        object: JsObject,
        children: Vec<WorkTreeNode<'a>>,
    },
}

impl<'a> WorkTreeNode<'a> {
    pub fn new_branch() -> Self {
        Self::Branch {
            object: JsObject::new(),
            children: Vec::new(),
        }
    }

    pub fn new_leaf(content: &'a str) -> Self {
        Self::Leaf {
            content,
            object: JsObject::new(),
        }
    }

    pub fn object(&self) -> &JsObject {
        match self {
            Self::Leaf { object, .. } => object,
            Self::Branch { object, .. } => object,
        }
    }

    pub fn children(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { children, .. } => Some(children),
        }
    }

    pub fn children_mut(&mut self) -> Option<&mut Vec<Self>> {
        match self {
            Self::Leaf { .. } => None,
            Self::Branch { children, .. } => Some(children),
        }
    }

    pub fn node_at_mut(&mut self, path: &[usize]) -> Option<&mut Self> {
        let mut node = self;

        for index in path {
            node = node.children_mut()?.get_mut(*index)?
        }

        Some(node)
    }

    pub fn walk<F>(&'a self, mut callback: F)
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
                    continue;
                }
            };

            callback(node, &path);

            *index += 1;
            *path.last_mut().unwrap() += 1;

            if let Some(node_children) = node.children() {
                nodes.push((node_children, 0));
                path.push(0);
            }
        }
    }
}
