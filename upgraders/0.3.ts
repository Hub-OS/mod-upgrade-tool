import {
  arraysEqual,
  ASTNode,
  collectTokens,
  findFiles,
  getArgumentNode,
  getMethodNameNode,
  parseLua54,
  patch,
  Patch,
  walkAst,
} from "../util.ts";

export const PREVIOUS_VERSION = "0.2";
export const NEXT_VERSION = "0.3";

const leafRewrites: { [key: string]: string } = {
  shortname: "short_name",
  include: "require",
  CardAction: "Action",
  card_action_event: "queue_action",
  get_actor: "get_owner",
  copy_metadata: "copy_card_properties",
  set_metadata: "set_card_properties",
  highlight_tile: "set_tile_highlight",
  reserve_entity_by_id: "reserve_for_id",
  Empty: "PermaHole",
  MoveAction: "Battle.Movement",
  raw_move_event: "queue_movement",
  get_current_palette: "get_palette",
  set_animation: "load_animation",
  show: "reveal",
  enable_parent_shader: "use_root_shader",
  refresh: "apply",
  reset_turn_gauge_to_default: "reset_turn_gauge_max_time",
  get_turn_gauge_value: "get_turn_gauge_progress",
  IntangibleRule: "Battle.IntangibleRule",
};

type MethodPatcher = {
  nameToken: string;
  patchFunction: (node: ASTNode, source: string) => Patch[] | undefined;
};

const method_patchers: MethodPatcher[] = [
  {
    nameToken: "is_cracked",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;

      return [
        new Patch(method_node.start, method_node.end, "get_state"),
        new Patch(node.end, node.end, " == TileState.Cracked"),
      ];
    },
  },
  {
    nameToken: "is_hidden",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;

      return [
        new Patch(method_node.start, method_node.end, "get_state"),
        new Patch(node.end, node.end, " == TileState.Hidden"),
      ];
    },
  },
  {
    nameToken: "is_hole",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;

      return [
        new Patch(node.start, node.start, "not "),
        new Patch(method_node.start, method_node.end, "is_walkable"),
      ];
    },
  },
  {
    nameToken: "shake_camera",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;
      const duration_arg = getArgumentNode(node, 1);

      if (!duration_arg) {
        return;
      }

      return [
        new Patch(method_node.start, method_node.end, "get_field():shake"),
        new Patch(duration_arg.end, duration_arg.end, " * 60"),
      ];
    },
  },
];

type FunctionPatcher = {
  nameTokens: string[];
  patchFunction: (node: ASTNode) => Patch[] | undefined;
};

const function_patchers: FunctionPatcher[] = [
  {
    nameTokens: ["HitProps", ".", "new"],
    patchFunction: (node) => {
      const name_node = node.children![0];

      return [new Patch(name_node.start, name_node.end, "Battle.HitProps.new")];
    },
  },
];

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  console.log("Moving mods/enemies/ to mods/battles/");

  try {
    await Deno.mkdir(mod_folder + "/battles/");
  } catch {
    // we don't care if this folder already exists.
  }

  for await (const entry of Deno.readDir(mod_folder + "/enemies")) {
    const source = mod_folder + "/enemies/" + entry.name;
    const dest = mod_folder + "/battles/" + entry.name;

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

        // function patches
        const prefix_exp_tokens = collectTokens(node.children[0]);

        for (const patcher of function_patchers) {
          if (!arraysEqual(prefix_exp_tokens, patcher.nameTokens)) {
            continue;
          }

          const function_patches = patcher.patchFunction(node);

          if (!function_patches) {
            console.log(`Failed to patch "${method_name}" in "${path}"`);
            continue;
          }

          patches.push(...function_patches);
        }
      }
    });

    let patched_source = source;

    // apply patches
    if (patches.length > 0) {
      console.log('Patching "' + path + '"...');
      patched_source = patch(source, patches);
    }

    if (patches.length > 0) {
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
