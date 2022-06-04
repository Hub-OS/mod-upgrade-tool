import init, { Lua54Parser, hook_panics } from './mod_parsers_wasm.js';
const wasm_path = new URL('mod_parsers_wasm_bg.wasm', import.meta.url);
await init(Deno.readFile(wasm_path));
hook_panics();
export { Lua54Parser };
