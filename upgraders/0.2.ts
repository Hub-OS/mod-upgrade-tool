// package.toml (meta files) introduced
// built referencing keristero's scraper work
// https://github.com/Keristero/onb-modlister/blob/master/package-scraper/scrape_package.js

import {
  createLuaEngine,
  findFiles,
  getParentFolder,
  getAncestorFolder,
  LuaEngine,
  TOML,
  parseLua54,
  walk,
  collectTokens,
  Patch,
  patch,
} from "../util.ts";

const leafRewrites: { [key: string]: string } = {
  mod_max_health: "boost_max_health",
};

type PackageMeta = {
  package: {
    category: string;
    id: string;
    name: string;
    description?: string;

    // blocks
    colors?: string[];
    shape?: number[][];
    flat?: boolean;

    // cards
    codes?: string[];
    long_description?: string;
    damage?: number;
    element?: string;
    secondary_element?: string;
    card_class?: string;
    limit?: number;
    hit_flags?: string[];
    can_boost?: boolean;
    counterable?: boolean;
    time_freeze?: boolean;
    skip_time_freeze_intro?: boolean;
    meta_classes?: string[];

    // players
    overworld_animation_path?: string;
    overworld_texture_path?: string;
    mugshot_animation_path?: string;
    mugshot_texture_path?: string;
    emotions_texture_path?: string;

    // cards, enemies, and players
    preview_texture_path?: string;

    // players and cards
    icon_texture_path?: string;
  };
  defines: {
    characters: { id: string; path: string }[];
  };
  dependencies: {
    characters: string[];
    libraries: string[];
    cards: string[];
  };
};

function injectGlobals(lua: LuaEngine, meta: PackageMeta, entry_path: string) {
  lua.global.set("_upgrader_folder_path", getParentFolder(entry_path));
  lua.global.set("include", (path: string) => {
    if (path[0] == ":") {
      path = path.slice(1);
    } else {
      path = getParentFolder(entry_path) + "/" + path;
    }

    const parent_folder = getParentFolder(path);
    const source =
      `local function include(path) return _G.include(":${parent_folder}/"..path) end _export = (function() ` +
      Deno.readTextFileSync(path) +
      "\nend)()";

    lua.doStringSync(source);

    return lua.global.get("_export");
  });

  lua.global.set("Engine", {
    load_texture: (path: string) => path,
    load_audio: (path: string) => path,
    define_character: (id: string, path: string) => {
      meta.defines.characters.push({ id, path });
    },
    // dependencies
    requires_character: (id: string) => {
      meta.dependencies.characters.push(id);
    },
    requires_card: (id: string) => {
      meta.dependencies.cards.push(id);
    },
    requires_library: (id: string) => {
      meta.dependencies.libraries.push(id);
    },
  });

  lua.global.set("Blocks", {
    White: "white",
    Red: "red",
    Green: "green",
    Blue: "blue",
    Pink: "pink",
    Yellow: "yellow",
  });

  lua.global.set("CardClass", {
    Standard: "standard",
    Mega: "mega",
    Giga: "giga",
    Dark: "dark",
  });

  lua.global.set("Element", {
    None: "none",
    Fire: "fire",
    Aqua: "aqua",
    Elec: "elec",
    Wood: "wood",
    Sword: "sword",
    Wind: "wind",
    Cursor: "cursor",
    Summon: "summon",
    Plus: "plus",
    Break: "break",
  });

  lua.global.set("Hit", {
    None: 0x00000000,
    RetainIntangible: 0x00000001,
    Freeze: 0x00000002,
    PierceInvis: 0x00000004,
    Flinch: 0x00000008,
    Shake: 0x00000010,
    Paralyze: 0x00000020,
    Stun: 0x00000020,
    Flash: 0x00000040,
    PierceGuard: 0x00000080,
    Impact: 0x00000100,
    Drag: 0x00000200,
    Bubble: 0x00000400,
    NoCounter: 0x00000800,
    Root: 0x00001000,
    Blind: 0x00002000,
    Confuse: 0x00004000,
    PierceGround: 0x00008000,
  });

  lua.global.set("Color", {
    new: (r: number, g: number, b: number, a: number) => {
      return { r, g, b, a };
    },
  });
}

function createPackageTable(meta: PackageMeta) {
  return {
    declare_package_id: (id: string) => {
      meta.package.id = id;
    },
    set_name: (name: string) => {
      meta.package.name = name;
    },
    set_description: (description: string) => {
      meta.package.description = description;
    },

    // blocks
    set_color: (color: string) => {
      meta.package.colors = [color];
    },
    set_shape: (shape: number[]) => {
      meta.package.shape = [[], [], [], [], []];

      for (let i = 0; i < shape.length; i++) {
        meta.package.shape[Math.floor(i / 5)][i % 5] = shape[i];
      }
    },
    set_mutator: () => {},
    as_program: () => {
      meta.package.flat = true;
    },

    // cards
    set_codes: (codes: string[]) => {
      meta.package.codes = codes;
    },
    get_card_props: () => {
      const card_props = {
        set shortname(shortname: string) {
          meta.package.name = shortname;
        },
        get shortname() {
          return meta.package.name;
        },
        set description(description: string) {
          meta.package.description = description;
        },
        set long_description(long_description: string) {
          meta.package.long_description = long_description;
        },
        set damage(damage: number) {
          meta.package.damage = damage;
        },
        set element(element: string) {
          meta.package.element = element;
        },
        set secondary_element(secondary_element: string) {
          meta.package.secondary_element = secondary_element;
        },
        set card_class(card_class: string) {
          meta.package.card_class = card_class;
        },
        set limit(limit: number) {
          meta.package.limit = limit;
        },
        set hit_flags(hit_flags: number) {
          meta.package.hit_flags = resolveHitFlags(hit_flags);
        },
        set can_boost(can_boost: boolean) {
          meta.package.can_boost = can_boost;
        },
        set counterable(counterable: boolean) {
          meta.package.counterable = counterable;
        },
        set time_freeze(time_freeze: boolean) {
          meta.package.time_freeze = time_freeze;
        },
        set skip_time_freeze_intro(skip_time_freeze_intro: boolean) {
          meta.package.skip_time_freeze_intro = skip_time_freeze_intro;
        },
        set meta_classes(meta_classes: string[]) {
          meta.package.meta_classes = meta_classes;
        },
      };

      // looks like wasmoon does take this as a reference as we need
      return card_props;
    },

    // players
    set_overworld_animation_path: (path: string) => {
      meta.package.overworld_animation_path = path;
    },
    set_overworld_texture_path: (path: string) => {
      meta.package.overworld_texture_path = path;
    },
    set_mugshot_texture_path: (path: string) => {
      meta.package.mugshot_texture_path = path;
    },
    set_mugshot_animation_path: (path: string) => {
      meta.package.mugshot_animation_path = path;
    },
    set_emotions_texture_path: (path: string) => {
      meta.package.emotions_texture_path = path;
    },

    // cards, enemies, and players
    set_preview_texture: (path: string) => {
      meta.package.preview_texture_path = path;
    },
    set_preview_texture_path: (path: string) => {
      meta.package.preview_texture_path = path;
    },

    // cards + players
    set_icon_texture: (path: string) => {
      meta.package.icon_texture_path = path;
    },
    set_icon_texture_path: (path: string) => {
      meta.package.icon_texture_path = path;
    },

    // todo: fix in 0.1.ts and remove this
    set_health: () => {},
  };
}

function resolveHitFlags(bitflags: number): string[] {
  const flags = [];

  const flag_names = [
    "none",
    "retain_intangible",
    "freeze",
    "pierce_invis",
    "flinch",
    "shake",
    "paralyze",
    "flash",
    "pierce_guard",
    "impact",
    "drag",
    "bubble",
    "no_counter",
    "root",
    "blind",
    "confuse",
    "pierce_ground",
  ];

  for (let i = 1; i < flag_names.length; i++) {
    if ((bitflags & (i ** 2)) != 0) {
      flags.push(flag_names[i]);
    }
  }

  return flags;
}

function resolvePackageCategory(entry_path: string): string {
  const category_folder = getAncestorFolder(entry_path, 1);
  const index = category_folder.lastIndexOf("/");

  if (index == -1) {
    return "";
  }

  switch (category_folder.slice(index + 1)) {
    case "blocks":
      return "block";
    case "cards":
      return "card";
    case "enemies":
      return "battle";
    case "players":
      return "player";
    case "libraries":
      return "library";
    default:
      return "";
  }
}

async function generateMetaFile(
  lua: LuaEngine,
  path: string,
  source: string
): Promise<boolean> {
  const meta: PackageMeta = {
    package: {
      category: resolvePackageCategory(path),
      id: "",
      name: "",
    },
    defines: {
      characters: [],
    },
    dependencies: {
      characters: [],
      libraries: [],
      cards: [],
    },
  };

  injectGlobals(lua, meta, path);
  await lua.doString(source);

  const package_table = createPackageTable(meta);

  lua.global.get("package_requires_scripts")?.(package_table);
  const package_init = lua.global.get("package_init");

  if (!package_init) {
    // can't generate a package.toml
    return false;
  }

  package_init(package_table);

  const meta_text = TOML.stringify(meta);

  const meta_path = getParentFolder(path) + "/package.toml";
  console.log(`Creating ${meta_path}`);

  await Deno.writeTextFile(meta_path, meta_text);

  return true;
}

async function stripPackageFunctions(path: string, source: string) {
  const ast = parseLua54(source);
  const patches: Patch[] = [];

  walk(ast, (node) => {
    const leafRewrite = node.content && leafRewrites[node.content];

    if (leafRewrite) {
      patches.push(new Patch(node.start, node.end, leafRewrite));
      return;
    }

    if (node.type == "stat" && node.children![0]!.content == "function") {
      const func_name = collectTokens(node.children![1]!).join("");

      if (
        func_name == "package_init" ||
        func_name == "package_requires_scripts"
      ) {
        patches.push({ start: node.start, end: node.end, content: "" });
      }
    }
  });

  if (patches.length == 0) {
    return;
  }

  console.log('Patching "' + path + '"...');
  await Deno.writeTextFile(path, patch(source, patches));
}

export const PREVIOUS_VERSION = "0.1";
export const NEXT_VERSION = "0.2";

export default async function (game_folder: string) {
  const mod_folder = game_folder + "/mods";
  const files = (await findFiles(mod_folder)).filter(
    (path: string) =>
      path.endsWith("entry.lua") && getAncestorFolder(path, 2) == mod_folder
  );

  for (const path of files) {
    const lua = await createLuaEngine();

    try {
      const source = await Deno.readTextFile(path);

      if (!(await generateMetaFile(lua, path, source))) {
        continue;
      }

      stripPackageFunctions(path, source);
    } catch (err) {
      console.error(`${err} in "${path}"`);
    } finally {
      lua.global.close();
    }
  }
}
