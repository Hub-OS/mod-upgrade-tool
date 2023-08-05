import {
  arraysEqual,
  collectTokens,
  findFiles,
  parseLua54,
  patch,
  Patch,
  walkAst,
} from "../util.ts";

export const PREVIOUS_VERSION = "0.3";
export const NEXT_VERSION = "0.9";

const renamed_enums: { tokens: string[]; new_content: string }[] = [
  {
    tokens: ["AudioPriority", ".", "Lowest"],
    new_content: "AudioBehavior.NoOverlap",
  },
  {
    tokens: ["AudioPriority", ".", "Low"],
    new_content: "AudioBehavior.Default",
  },
  {
    tokens: ["AudioPriority", ".", "High"],
    new_content: "AudioBehavior.Default",
  },
  {
    tokens: ["AudioPriority", ".", "Highest"],
    new_content: "AudioBehavior.Default",
  },
];

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
      for (const { tokens, new_content } of renamed_enums) {
        if (!source.startsWith(tokens[0], node.start)) {
          continue;
        }

        const node_tokens = collectTokens(node);

        if (arraysEqual(tokens, node_tokens)) {
          patches.push(new Patch(node.start, node.end, new_content));
          console.log(source.slice(node.start, node.end), new_content);
          console.log(node.start, node.end);
        }
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
