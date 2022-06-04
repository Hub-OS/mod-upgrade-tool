use crate::work_tree::WorkTreeNode;
use ebnf_parser::ASTNode;
use js_sys::Object as JsObject;
use js_sys::Reflect as JsReflect;
use wasm_bindgen::prelude::*;

pub fn ast_to_js_object<'a>(ast: ASTNode<'a, &'a str>) -> Result<JsObject, JsValue> {
    let mut work_root = WorkTreeNode::new_branch();

    // first pass for creating the tree
    let type_js_str = JsValue::from_str("type");
    let start_js_str = JsValue::from_str("start");
    let end_js_str = JsValue::from_str("end");

    ast.walk(|node, path| {
        let node_object = if path.is_empty() {
            work_root.object()
        } else {
            let work_node_parent = work_root.node_at_mut(&path[..path.len() - 1]).unwrap();
            let children = work_node_parent.children_mut().unwrap();

            match node {
                ASTNode::Branch { .. } => children.push(WorkTreeNode::new_branch()),
                ASTNode::Leaf { token, .. } => children.push(WorkTreeNode::new_leaf(token.content)),
            };

            children.last().unwrap().object()
        };

        let start = &JsValue::from_f64(node.start() as f64);
        let end = &JsValue::from_f64(node.end() as f64);

        JsReflect::set(node_object, &type_js_str, &JsValue::from_str(node.label())).unwrap();
        JsReflect::set(node_object, &start_js_str, start).unwrap();
        JsReflect::set(node_object, &end_js_str, end).unwrap();
    });

    // second pass
    // add enum type specific properties
    use js_sys::Array as JsArray;

    let children_js_str = JsValue::from_str("children");
    let content_js_str = JsValue::from_str("content");

    work_root.walk(|work_node, _| {
        match work_node {
            WorkTreeNode::Branch { object, children } => {
                let children_obj =
                    JsArray::from_iter(children.iter().map(|child_node| child_node.object()));

                JsReflect::set(object, &children_js_str, &children_obj).unwrap();
            }
            WorkTreeNode::Leaf { object, content } => {
                JsReflect::set(object, &content_js_str, &JsValue::from_str(content)).unwrap();
            }
        };
    });

    match work_root {
        WorkTreeNode::Leaf { object, .. } => Ok(object),
        WorkTreeNode::Branch { object, .. } => Ok(object),
    }
}
