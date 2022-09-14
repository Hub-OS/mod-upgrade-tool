mod ast_to_js;
mod work_tree;

use crate::ast_to_js::ast_to_js_object;
use js_sys::Object as JsObject;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn hook_panics() {
    use console_error_panic_hook;
    use std::panic;

    panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct Lua54Parser {
    parser: lua_parsers::Lua54Parser,
}

#[wasm_bindgen]
impl Lua54Parser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Lua54Parser, JsValue> {
        Ok(Self {
            parser: lua_parsers::Lua54Parser::new(),
        })
    }

    pub fn parse(&self, source: &str) -> Result<JsObject, JsValue> {
        let ast = self
            .parser
            .parse(source)
            .map_err(|err| format!("{:#}", err))?;

        ast_to_js_object(ast)
    }
}
