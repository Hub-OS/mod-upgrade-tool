import {
  arraysEqual,
  collectTokens,
  findFiles,
  parseLua54,
  patch,
  Patch,
  walk,
} from "../util.ts";

const leafRewrites: { [key: string]: string } = {
  // callbacks
  update_func: "on_update_func",
  delete_func: "on_delete_func",
  draw_func: "on_draw_func",
  collision_func: "on_collision_func",
  replace_func: "on_replace_func",
  attack_func: "on_attack_func",
  battle_start_func: "on_battle_start_func",
  battle_end_func: "on_battle_end_func",
  spawn_func: "on_spawn_func",
  animation_end_func: "on_animation_end_func",
  execute_func: "on_execute_func",
  action_end_func: "on_action_end_func",
  intro_func: "on_intro_func",
  countered_func: "on_countered_func",
  scene_inject_func: "on_scene_inject_func",
  // "Hit.*"
  Pierce: "PierceInvis",
  Breaking: "PierceGuard",
  // Animation:point() -> Animation:get_point, might rewrite variables named point
  point: "get_point",
};

const branchRewrites = [
  // Hit.Retangible was deleted
  { type: "exp", tokens: ["Hit", ".", "Retangible"], content: "0" },
];

const createDefenseRuleTokens = ["Battle", ".", "DefenseRule", ".", "new"];

export const PREVIOUS_VERSION = "v2";
export const NEXT_VERSION = "v2.5";

export default async function (game_folder: string) {
  const files = await findFiles(game_folder + "/resources");

  const luaFiles = files.filter((path: string) =>
    path.toLowerCase().endsWith(".lua")
  );

  for (const path of luaFiles) {
    const source = await Deno.readTextFile(path);
    let ast;

    try {
      ast = parseLua54(source);
    } catch (e) {
      console.log('Failed to parse "' + path + '":\n', e);
      continue;
    }

    const patches: Patch[] = [];
    let contains_frame_data = false;
    let contains_frame_data_patch = false;

    walk(ast, (node) => {
      contains_frame_data ||= node.content == "make_frame_data";
      contains_frame_data_patch ||= node.content == "old_make_frame_data";

      const leafRewrite = node.content && leafRewrites[node.content];

      if (leafRewrite) {
        patches.push(new Patch(node.start, node.end, leafRewrite));
        return;
      }

      if (!node.children) {
        // remaining patches are for branches
        return;
      }

      if (
        node.type == "functioncall" &&
        arraysEqual(collectTokens(node.children[0]), createDefenseRuleTokens)
      ) {
        // swapping the first argument with DefensePriority.Last
        const argsNode = node.children[node.children.length - 1];
        const expListNode = argsNode.children?.[1];
        const firstArgNode = expListNode?.children?.[0];

        if (firstArgNode) {
          patches.push(
            new Patch(
              firstArgNode.start,
              firstArgNode.end,
              "DefensePriority.Last"
            )
          );
        }

        return;
      }

      const rangeRenames = branchRewrites.filter(
        (rename) => rename.type == node.type
      );

      if (rangeRenames.length == 0) {
        // nothing to do, move on to the next node
        return;
      }

      const tokens = collectTokens(node);

      for (const rename of rangeRenames) {
        if (arraysEqual(tokens, rename.tokens)) {
          patches.push(new Patch(node.start, node.end, rename.content));
          return;
        }
      }
    });

    if (!contains_frame_data_patch && contains_frame_data) {
      patches.push({
        start: 0,
        end: 0,
        content: `\
local old_make_frame_data = make_frame_data
local function make_frame_data(frames)
  local updated_frames = {}
  for i, pair in ipairs(frames) do
    updated_frames[i] = { pair[1], math.floor(pair[2] * 60 + 0.5) }
  end
  return old_make_frame_data(updated_frames)
end

`,
      });
    }

    if (patches.length > 0) {
      console.log('Patching "' + path + '"...');
      await Deno.writeTextFile(path, patch(source, patches));
    }
  }
}
