use std::fs;
use std::process::Command;

fn main() {
    let build_output = Command::new("wasm-pack")
        .args(["build", "--target", "web"])
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .output()
        .unwrap();

    if !build_output.status.success() {
        // stdout + stderr are shared, no need to display anything
        return;
    }

    let pkg_name = env!("CARGO_PKG_NAME").replace('-', "_");

    let index_src = format!(
        "\
import init, {{ Lua54Parser, hook_panics }} from './{}.js';
const wasm_path = new URL('{}_bg.wasm', import.meta.url);
await init(Deno.readFile(wasm_path));
hook_panics();
export {{ Lua54Parser }};
",
        pkg_name, pkg_name
    );

    fs::write("pkg/index.ts", index_src).unwrap();
}
