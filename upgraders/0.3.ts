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
  TOML,
} from "../util.ts";

export const PREVIOUS_VERSION = "0.2";
export const NEXT_VERSION = "0.3";

const leafRewrites: { [key: string]: string } = {
  shortname: "short_name",
  include: "require",
  CardAction: "Action",
  card_action_event: "queue_action",
  get_actor: "owner",
  copy_metadata: "copy_card_properties",
  set_metadata: "set_card_properties",
  highlight_tile: "set_tile_highlight",
  reserve_entity_by_id: "reserve_for_id",
  Empty: "PermaHole",
  MoveAction: "Movement",
  raw_move_event: "queue_movement",
  get_current_palette: "palette",
  set_animation: "load_animation",
  show: "reveal",
  enable_parent_shader: "use_root_shader",
  refresh: "apply",
  Battlestep: "Battle",
  toggle_counter: "set_counterable",
  toggle_hitbox: "enable_hitbox",
  share_tile: "enable_sharing_tile",
  battle_init: "encounter_init",
  Engine: "Resources",
  get_id: "id",
  get_owner: "owner",
  get_state: "state",
  get_level: "level",
  get_field: "field",
  get_name: "name",
  get_element: "element",
  get_facing: "facing",
  get_team: "team",
  get_current_tile: "current_tile",
  get_offset: "offset",
  get_tile_offset: "tile_offset",
  get_elevation: "elevation",
  get_texture: "texture",
  get_color: "color",
  get_animation: "animation",
  get_context: "context",
  get_rank: "rank",
  get_max_health: "max_health",
  get_health: "health",
  get_attack_level: "attack_level",
  get_rapid_level: "rapid_level",
  get_charge_level: "charge_level",
  get_animation_progress: "animation_progress",
  get_layer: "layer",
  get_origin: "origin",
  get_scale: "scale",
  get_size: "size",
  get_width: "width",
  get_height: "height",
  get_color_mode: "color_mode",
  get_palette: "palette",
  is_sharing_tile: "sharing_tile",
  is_deleted: "deleted",
  is_counterable: "counterable",
  is_intangible: "intangible",
  is_visible: "visible",
  is_replaced: "replaced",
  is_damage_blocked: "damage_blocked",
  is_impact_blocked: "impact_blocked",
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
        new Patch(method_node.start, method_node.end, "state"),
        new Patch(node.end, node.end, " == TileState.Cracked"),
      ];
    },
  },
  {
    nameToken: "is_hidden",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;

      return [
        new Patch(method_node.start, method_node.end, "state"),
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
  {
    nameToken: "stream_music",
    patchFunction: function (node, _) {
      const method_node = getMethodNameNode(node)!;

      return [new Patch(method_node.start, method_node.end, "set_music")];
    },
  },
];

type FunctionPatcher = {
  nameTokens?: string[];
  patchFunction: (node: ASTNode) => Patch[] | undefined;
};

function generateRenameFuncPatchFunc(name: string): (node: ASTNode) => [Patch] {
  return function (function_node) {
    const name_node = function_node.children![0];

    return [new Patch(name_node.start, name_node.end, name)];
  };
}

const function_patchers: FunctionPatcher[] = [
  {
    nameTokens: ["Engine", ".", "stream_music"],
    patchFunction: generateRenameFuncPatchFunc("Resources.play_music"),
  },
  {
    nameTokens: ["Engine", ".", "get_turn_gauge_value"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.progress"),
  },
  {
    nameTokens: ["Engine", ".", "get_turn_gauge_time"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.time"),
  },
  {
    nameTokens: ["Engine", ".", "set_turn_gauge_time"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.set_time"),
  },
  {
    nameTokens: ["Engine", ".", "get_turn_gauge_max_time"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.max_time"),
  },
  {
    nameTokens: ["Engine", ".", "set_turn_gauge_max_time"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.set_max_time"),
  },
  {
    nameTokens: ["Engine", ".", "reset_turn_gauge_to_default"],
    patchFunction: generateRenameFuncPatchFunc("TurnGauge.reset_max_time"),
  },
  {
    // remove `Battle.`
    patchFunction: (node) => {
      const name_node = node.children![0];

      const first_node = name_node.children![0];
      const second_node = name_node.children![1];

      if (first_node?.content != "Battle" || second_node?.content != ".") {
        return [];
      }

      return [new Patch(first_node.start, second_node.end, "")];
    },
  },
];

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";

  console.log("Moving mods/enemies/ to mods/encounters/");

  try {
    await Deno.mkdir(mod_folder + "/encounters/");
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
  const tomlFiles = files.filter((path) =>
    path.toLowerCase().endsWith("package.toml")
  );

  for (const path of tomlFiles) {
    const toml = await Deno.readTextFile(path);

    const tomlObject = TOML.parse(toml);
    let modified = false;

    // update category
    if (tomlObject.category == "battle") {
      tomlObject.category = "encounter";
      modified = true;
    }

    // update dependencies
    const dependencies = tomlObject.dependencies as
      | { [key: string]: unknown }
      | undefined;

    if (
      dependencies &&
      typeof dependencies == "object" &&
      dependencies.battles
    ) {
      dependencies.encounters = dependencies.battles;
      delete dependencies.battles;
      modified = true;
    }

    // save toml
    if (modified) {
      console.log('Patching "' + path + '"...');
      await Deno.writeTextFile(path, TOML.stringify(tomlObject));
    }
  }

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
          if (
            patcher.nameTokens &&
            !arraysEqual(prefix_exp_tokens, patcher.nameTokens)
          ) {
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
