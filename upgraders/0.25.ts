import { Patch, findFiles, parseLua54, patch, walkAst } from "../util.ts";

export const PREVIOUS_VERSION = "0.20";
export const NEXT_VERSION = "0.25";

const leafRewrites: { [key: string]: string } = {
  can_block_func: "defense_func",
  filter_statuses_func: "filter_func",
};

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  const files = await findFiles(mod_folder);
  const luaFiles = files.filter((path) => path.toLowerCase().endsWith(".lua"));

  for (const path of luaFiles) {
    const source = await Deno.readTextFile(path);

    let ast;

    try {
      ast = parseLua54(source);
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
    if (patches.length > 0) {
      console.log('Patching "' + path + '"...');
      const patched_source = patch(source, patches);
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
