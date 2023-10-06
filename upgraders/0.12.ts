import {
  ASTNode,
  findFiles,
  getMethodNameNode,
  parseLua54,
  patch,
  Patch,
  walkAst,
} from "../util.ts";

export const PREVIOUS_VERSION = "0.11";
export const NEXT_VERSION = "0.12";

const leafRewrites: { [key: string]: string } = {
  Lifetimes: "Lifetime",
};

type MethodPatcher = {
  nameToken: string;
  patchFunction: (node: ASTNode, source: string) => Patch[] | undefined;
};

function createFunctionRenamePatcher(
  new_name: string
): (node: ASTNode, source: string) => Patch[] {
  return function (node, _) {
    const nameNode = getMethodNameNode(node)!;

    return [new Patch(nameNode.start, nameNode.end, new_name)];
  };
}

const method_patchers: MethodPatcher[] = [
  {
    nameToken: "get_augments",
    patchFunction: createFunctionRenamePatcher("set_highlight"),
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
      const leafRewrite = node.content && leafRewrites[node.content];

      if (leafRewrite) {
        patches.push(new Patch(node.start, node.end, leafRewrite));
        return;
      }

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
