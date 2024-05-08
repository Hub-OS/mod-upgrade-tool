import { findFiles } from "../util.ts";

export const PREVIOUS_VERSION = "0.13";
export const NEXT_VERSION = "0.20";

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  const files = await findFiles(mod_folder);
  const luaFiles = files.filter((path) => path.toLowerCase().endsWith(".lua"));

  for (const path of luaFiles) {
    const source = await Deno.readTextFile(path);

    let patched_source = source.replace(/TileState\.Hidden/g, "TileState.Void");

    // apply patches
    if (source != patched_source) {
      console.log('Patching "' + path + '"...');
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
