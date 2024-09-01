import { Patch, findFiles, parseLua54, patch, walkAst } from "../util.ts";

export const PREVIOUS_VERSION = "0.25";
export const NEXT_VERSION = "0.27";

const leafRewrites: { [key: string]: string } = {
  require_hit_flag: "require_hit_flags",
};

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  const files = await findFiles(mod_folder);

  for (const path of files) {
    const lowercase_path = path.toLowerCase();

    const is_lua = lowercase_path.endsWith(".lua");
    const is_animation = lowercase_path.endsWith(".animation");

    if (!is_lua && !is_animation) {
      continue;
    }

    const source = await Deno.readTextFile(path);

    let patched_source = source
      .replaceAll('"PLAYER_SHOOTING"', '"CHARACTER_SHOOT"')
      .replaceAll('"PLAYER_SWORD"', '"CHARACTER_SWING"')
      .replaceAll('"PLAYER_', '"CHARACTER_');

    const patches: Patch[] = [];

    if (is_lua) {
      let ast;

      try {
        ast = parseLua54(patched_source);
      } catch (e) {
        console.error(`%cFailed to parse "${path}":\n${e}`, "color: red");
        continue;
      }

      walkAst(ast, (node) => {
        const leafRewrite = node.content && leafRewrites[node.content];

        if (leafRewrite) {
          patches.push(new Patch(node.start, node.end, leafRewrite));
          return;
        }
      });
    }

    // apply patches
    if (source != patched_source || patches.length > 0) {
      console.log('Patching "' + path + '"...');
      patched_source = patch(patched_source, patches);
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
