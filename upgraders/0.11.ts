import {
  ASTNode,
  findFiles,
  getArgumentNode,
  getMethodNameNode,
  parseLua54,
  patch,
  Patch,
  walkAst,
} from "../util.ts";

export const PREVIOUS_VERSION = "0.10";
export const NEXT_VERSION = "0.11";

type MethodPatcher = {
  nameToken: string;
  patchFunction: (node: ASTNode, source: string) => Patch[] | undefined;
};

function patchFindHittableEntitiesMethod(node: ASTNode, _: string) {
  const arg_node = getArgumentNode(node, 0)!;

  return [
    new Patch(
      arg_node.start,
      arg_node.start,
      "--[[hittable patch--]]function(entity) if not entity:hittable() then return end (--[[end hittable patch--]]"
    ),
    new Patch(
      arg_node.end,
      arg_node.end,
      "--[[hittable patch--]])(entity) end--[[end hittable patch--]]"
    ),
  ];
}

const method_patchers: MethodPatcher[] = [
  {
    nameToken: "find_entities",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "find_characters",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "find_obstacles",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "find_players",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "find_nearest_characters",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "find_nearest_players",
    patchFunction: patchFindHittableEntitiesMethod,
  },
  {
    nameToken: "get_augments",
    patchFunction: function (node, _) {
      const nameNode = getMethodNameNode(node)!;

      return [new Patch(nameNode.start, nameNode.end, "augments")];
    },
  },
];

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  console.log("Moving mods/enemies/ to mods/encounters/");

  try {
    await Promise.allSettled([
      Deno.mkdir(mod_folder + "/encounters/"),
      Deno.mkdir(mod_folder + "/enemies/"),
    ]);
  } catch {
    // we don't care if this folder already exists.
  }

  for await (const entry of Deno.readDir(mod_folder + "/enemies")) {
    const source = mod_folder + "/enemies/" + entry.name;
    const dest = mod_folder + "/encounters/" + entry.name;

    try {
      await Deno.rename(source, dest);
    } catch {
      console.error(`%cFailed to move ${source} to ${dest}`, "color: red");
    }
  }

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
      if (!node.children) {
        // remaining patches are for branches
        return;
      }

      if (node.type == "functioncall") {
        // method patches
        const method_node = getMethodNameNode(node);
        const method_name = method_node?.content;

        for (const patcher of method_patchers) {
          if (patcher.nameToken != method_name) {
            continue;
          }

          const function_patches = patcher.patchFunction(node, source);

          if (!function_patches) {
            console.log(`Failed to patch "${method_name}" in "${path}"`);
            continue;
          }

          patches.push(...function_patches);
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
