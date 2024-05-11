import { Patch, findFiles, parseLua54, patch, walkAst } from "../util.ts";

export const PREVIOUS_VERSION = "0.13";
export const NEXT_VERSION = "0.20";

const leafRewrites: { [key: string]: string } = {
  tile_offset: "movement_offset",
  set_tile_offset: "set_movement_offset",
};

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  const files = await findFiles(mod_folder);
  const luaFiles = files.filter((path) => path.toLowerCase().endsWith(".lua"));

  for (const path of luaFiles) {
    const source = await Deno.readTextFile(path);

    let patched_source = source.replace(/TileState\.Hidden/g, "TileState.Void");

    let ast;

    try {
      ast = parseLua54(patched_source);
    } catch (e) {
      console.error(`%cFailed to parse "${path}":\n${e}`, "color: red");
      continue;
    }

    const patches: Patch[] = [];

    walkAst(ast, (node) => {
      const leafRewrite = node.content && leafRewrites[node.content];

      if (leafRewrite) {
        patches.push(new Patch(node.start, node.end, leafRewrite));
        return;
      }
    });

    // apply patches
    if (source != patched_source || patches.length > 0) {
      console.log('Patching "' + path + '"...');
      patched_source = patch(patched_source, patches);
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
