import {
  collectTokens,
  getArgumentNode,
  findFiles,
  parseLua54,
  patch,
  Patch,
  walkAst,
  ASTNode,
  getMethodNameNode,
  arraysEqual,
  getFunctionParameters,
  getParentFolder,
} from "../util.ts";

const leafRewrites: { [key: string]: string } = {
  // callbacks
  _folderpath: '""',
  set_special_description: "set_description",
  package_build: "battle_init",
  charged_time_table_func: "calculate_charge_time_func",
  card_create_action: "card_init",
};

type MethodPatcher = {
  nameToken: string;
  patchFunction: (node: ASTNode, source: string) => Patch[] | undefined;
};

// converts `package:set_*_texture(Engine.load_texture(""))`  to `package:set_*_texture_path("")`
function pathPatches(node: ASTNode): Patch[] | undefined {
  const texture_exp = getArgumentNode(node, 0);
  const texture_arg = texture_exp?.children?.[0].children?.[0];

  if (!texture_arg || texture_arg.type != "functioncall") {
    return undefined;
  }

  const pathNode = getArgumentNode(texture_arg, 0);

  if (!pathNode) {
    return undefined;
  }

  const method_node = getMethodNameNode(node)!;

  return [
    new Patch(
      method_node.start,
      method_node.end,
      method_node.content! + "_path"
    ),
    new Patch(texture_exp.start, pathNode.start, ""),
    new Patch(pathNode.end, texture_exp.end, ""),
  ];
}

function commentPatch(node: ASTNode): Patch[] {
  return [
    new Patch(node.start, node.start, "--[[ "),
    new Patch(node.end, node.end, " --]]"),
  ];
}

const method_patchers: MethodPatcher[] = [
  {
    nameToken: "set_speed",
    patchFunction: commentPatch,
  },
  {
    nameToken: "set_attack",
    patchFunction: commentPatch,
  },
  {
    nameToken: "set_charged_attack",
    patchFunction: commentPatch,
  },
  {
    nameToken: "set_icon_texture",
    patchFunction: pathPatches,
  },
  {
    nameToken: "set_preview_texture",
    patchFunction: pathPatches,
  },
  {
    nameToken: "set_offset",
    patchFunction: (node) => {
      const patches = [];

      const x_node = getArgumentNode(node, 0);
      const y_node = getArgumentNode(node, 1);

      if (x_node) {
        patches.push(new Patch(x_node.end, x_node.end, " * 0.5"));
      }

      if (y_node) {
        patches.push(new Patch(y_node.end, y_node.end, " * 0.5"));
      }

      return patches;
    },
  },
  {
    nameToken: "set_mutator",
    patchFunction: (node, source) => {
      const patches = [new Patch(node.start, node.end, "")];

      const arg = getArgumentNode(node, 0);

      if (arg) {
        patches.push(
          new Patch(
            source.length,
            source.length,
            "\naugment_init = function(augment) (" +
              source.slice(arg.start, arg.end) +
              ")(augment:get_owner()) end"
          )
        );
      }

      return patches;
    },
  },
];

type FunctionPatcher = {
  nameTokens: string[];
  patchFunction: (node: ASTNode) => Patch[] | undefined;
};

const make_frame_data_patcher: FunctionPatcher = {
  nameTokens: ["make_frame_data"],
  patchFunction: (node) => {
    return [
      new Patch(
        node.start,
        node.end,
        collectTokens(node.children![node.children!.length - 1]).join("")
      ),
    ];
  },
};

const default_function_patchers: FunctionPatcher[] = [
  {
    nameTokens: ["frames"],
    patchFunction: (node) => {
      return [
        new Patch(
          node.start,
          node.end,
          collectTokens(node.children![node.children!.length - 1]).join("")
        ),
      ];
    },
  },
  {
    nameTokens: ["make_async_lockout"],
    patchFunction: (node) => {
      const arg_node = getArgumentNode(node, 0);

      if (!arg_node) {
        return [];
      }

      // math.floor([ARG_NODE] * 60 + 0.5)
      return [
        new Patch(arg_node.start, arg_node.start, "math.floor("),
        new Patch(arg_node.end, arg_node.end, " * 60 + 0.5)"),
      ];
    },
  },
];

type SetterPatcher = {
  nameToken: string;
  expPatchFunction: (node: ASTNode) => Patch[] | undefined;
};

const setter_patchers: SetterPatcher[] = [
  {
    nameToken: "on_update_func",
    expPatchFunction: (exp_list_node) => {
      if (arraysEqual(collectTokens(exp_list_node), ["nil"])) {
        return [];
      }

      const exp_node = exp_list_node.children![0];
      const possible_function_def = exp_node.children![0];

      if (
        possible_function_def.type == "functiondef" &&
        getFunctionParameters(possible_function_def).length < 2
      ) {
        // this on_update_func isn't using delta time, no need to modify
        return [];
      }

      return [
        new Patch(
          exp_list_node.start,
          exp_list_node.start,
          " --[[dt patch--]] function(_upgrader_self) local onb_update_func = --[[end dt patch--]] "
        ),
        new Patch(
          exp_list_node.end,
          exp_list_node.end,
          " --[[dt patch--]] if onb_update_func then onb_update_func(_upgrader_self, 0.01666) end end --[[end dt patch--]]"
        ),
      ];
    },
  },
  {
    nameToken: "on_delete_func",
    expPatchFunction: (exp_list_node) => {
      const tokens = collectTokens(exp_list_node);

      if (arraysEqual(tokens, ["nil"])) {
        return [];
      }

      const exp_node = exp_list_node.children![0];
      const possible_function_def = exp_node.children![0];
      const parameter_nodes =
        possible_function_def.type == "functiondef"
          ? getFunctionParameters(possible_function_def)
          : undefined;

      if (parameter_nodes && parameter_nodes.length > 0) {
        const function_body = possible_function_def.children![1];
        const end_node =
          function_body.children![function_body.children!.length - 1];

        const entity_parameter = parameter_nodes[0].content;
        const erase_call = `${entity_parameter}:erase()`;

        if (tokens.join("").includes(erase_call)) {
          // no need to patch, function already calls erase
          return [];
        }

        return [
          new Patch(
            end_node.start,
            end_node.start,
            ` --[[patch--]] ${erase_call} --[[end patch--]] `
          ),
        ];
      }

      return [
        new Patch(
          exp_list_node.start,
          exp_list_node.start,
          " --[[patch--]] function(_upgrader_entity) local onb_delete_func = --[[end patch--]] "
        ),
        new Patch(
          exp_list_node.end,
          exp_list_node.end,
          " --[[patch--]] if onb_delete_func then onb_delete_func(_upgrader_entity) _upgrader_entity:erase() end end --[[end patch--]]"
        ),
      ];
    },
  },
];

function isCharacter(mod_folder: string, path: string): boolean {
  // is in the enemies folder + is not a root package
  const enemies_folder = mod_folder + "/enemies";
  return (
    path.includes(enemies_folder) &&
    getParentFolder(getParentFolder(path)) != enemies_folder
  );
}

function transformPath(
  path: string,
  new_mod_folder: string,
  old_mod_folder: string
): string {
  const category_end = path.indexOf("/", old_mod_folder.length + 1);
  let category_folder = path.slice(old_mod_folder.length + 1, category_end);

  if (category_folder == "blocks") {
    category_folder = "augments";
  }

  return new_mod_folder + "/" + category_folder + path.slice(category_end);
}

export const PREVIOUS_VERSION = "ONB-v2.5";
export const NEXT_VERSION = "0.1";

export default async function (game_folder: string) {
  const old_mod_folder = game_folder + "/resources/mods";
  const new_mod_folder = game_folder + "/mods";
  const files = await findFiles(old_mod_folder);

  console.log("Copying files to new mod folder...");

  for (const path of files) {
    const new_path = transformPath(path, new_mod_folder, old_mod_folder);
    const parent_folder = new_path.slice(0, new_path.lastIndexOf("/"));

    await Deno.mkdir(parent_folder, { recursive: true }).catch(
      // ignore error
      () => {}
    );

    await Deno.copyFile(path, new_path).catch(() => {
      console.log(`Failed to copy "${path}"`);
    });
  }

  const luaFiles = files
    .filter((path) => path.toLowerCase().endsWith(".lua"))
    .map((old_path) => transformPath(old_path, new_mod_folder, old_mod_folder));

  for (const path of luaFiles) {
    let source = await Deno.readTextFile(path);

    if (isCharacter(new_mod_folder, path)) {
      // child package of a battle, assuming to be a character
      source = source.replace(
        "function package_init",
        "function character_init"
      );
    }

    const has_v2_frame_data_patch = source.includes("old_make_frame_data");

    const function_patchers: FunctionPatcher[] = [...default_function_patchers];

    if (!has_v2_frame_data_patch) {
      function_patchers.push(make_frame_data_patcher);
    }

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
      } else if (node.type == "stat" && node.children[1]?.content == "=") {
        // setter patches
        const prefix_exp_tokens = collectTokens(node.children[0]);
        const property_name = prefix_exp_tokens[prefix_exp_tokens.length - 1];

        for (const patcher of setter_patchers) {
          if (property_name != patcher.nameToken) {
            continue;
          }

          const exp_patches = patcher.expPatchFunction(node.children[2]);

          if (!exp_patches) {
            console.log(`Failed to patch "${property_name}" in "${path}"`);
            continue;
          }

          patches.push(...exp_patches);
        }
      }
    });

    let patched_source = source;

    // apply patches
    if (patches.length > 0) {
      console.log('Patching "' + path + '"...');
      patched_source = patch(source, patches);
    }

    // fix _modpath
    const contains_modpath = patched_source.includes("_modpath");

    if (patched_source.includes("_modpath")) {
      const up_folder_count =
        path.slice(old_mod_folder.length).split("/").length - 3;

      let modpath_replacement = '""';

      if (up_folder_count > 0) {
        const suffix = "../".repeat(Math.max(up_folder_count, 0));

        modpath_replacement = `"${suffix}"`;
      }

      patched_source = patched_source.replaceAll(
        "_modpath",
        modpath_replacement
      );
    }

    if (has_v2_frame_data_patch) {
      patched_source = patched_source.replace(
        "local old_make_frame_data = make_frame_data",
        "local old_make_frame_data = function(data) return data end"
      );
    }

    if (
      (source.includes("add_step") || source.includes("Battle.Step.new")) &&
      !source.includes("Battle.Step.new = ")
    ) {
      patched_source =
        `\
-- Battle.Step.new() + card_action:add_step(step) Shims
-- delete and upgrade to card_action:create_step()
Battle.Step.new = function()
  return {}
end
Battle.CardAction.add_step = function(self, step)
  local real_step = self:create_step()
  for k, v in pairs(step) do
    real_step[k] = v
  end
  local forward = {
    __index = function(t, k) return real_step[k] end,
    __new_index = function(t, k, v) real_step[k] = v end
  }
  setmetatable(step, forward)
end
-- End of Shim

` + patched_source;
    }

    if (
      (source.includes("register_component") ||
        source.includes("Battle.Component.new")) &&
      !source.includes("Battle.Component.new = ")
    ) {
      patched_source =
        `\
-- Battle.Component.new(entity, lifetime) + entity:register_component(component) Shims
-- delete and upgrade to entity:create_component(lifetime)
Battle.Component.new = function(_, lifetime)
  return { ["*lifetime"] = lifetime }
end
Battle.Entity.register_component = function(self, component)
  local real_component = self:create_component(component["*lifetime"])
  component["*lifetime"] = nil
  for k, v in pairs(component) do
    real_component[k] = v
  end
  local forward = {
    __index = function(t, k) return real_component[k] end,
    __new_index = function(t, k, v) real_component[k] = v end
  }
  setmetatable(component, forward)
end
-- End of Shim

` + patched_source;
    }

    // saving
    if (patches.length > 0 || contains_modpath || has_v2_frame_data_patch) {
      await Deno.writeTextFile(path, patched_source);
    }
  }
}
