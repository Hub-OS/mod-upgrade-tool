export type Upgrader = {
  PREVIOUS_VERSION: string;
  NEXT_VERSION: string;
  default: (game_folder: string) => Promise<void>;
};

export class UpgradeTree {
  upgraders: Upgrader[] = [];

  resolveUpgradePath(
    start_version: string,
    end_version?: string
  ): Upgrader[] | null {
    let work_paths = this.upgraders
      .filter((upgrader) => upgrader.PREVIOUS_VERSION == start_version)
      .map((upgrader) => [upgrader]);
    let next_paths = [];

    while (work_paths.length > 0) {
      console.log(work_paths);
      for (const path of work_paths) {
        const latest = path[path.length - 1];

        if (latest.NEXT_VERSION == end_version) {
          return path;
        }

        next_paths.push(
          ...this.upgraders
            .filter(
              (upgrader) => upgrader.PREVIOUS_VERSION == latest.NEXT_VERSION
            )
            .map((upgrader) => [...path, upgrader])
        );
      }

      work_paths = next_paths;
      next_paths = [];
    }

    return null;
  }

  async upgrade(
    game_folder: string,
    start_version: string,
    end_version?: string
  ): Promise<boolean> {
    const upgraders = this.resolveUpgradePath(start_version, end_version);

    if (!upgraders) {
      console.error(
        `%cCould not find upgrade path from ${start_version} to ${end_version}`,
        "color: red"
      );
      return false;
    }

    console.log(`Upgrading from ${start_version} to ${end_version}`);

    // time to upgrade!
    for (const upgrader of upgraders) {
      console.log(
        `%cRunning upgrader for ${upgrader.PREVIOUS_VERSION} -> ${upgrader.NEXT_VERSION}`,
        "color: green"
      );
      await upgrader.default(game_folder);
    }

    return true;
  }
}

export async function loadUpgradeTree(): Promise<UpgradeTree> {
  const tree = new UpgradeTree();

  // find upgraders
  for await (const file of Deno.readDir("upgraders")) {
    if (!file.name.toLowerCase().endsWith(".ts")) {
      // not a typescript file
      continue;
    }

    const upgrader = await import("../upgraders/" + file.name);

    if (!upgrader.PREVIOUS_VERSION) {
      throw `${file.name} is missing PREVIOUS_VERSION definition`;
    }

    if (!upgrader.default || typeof upgrader.default != "function") {
      throw `${file.name} is missing default function`;
    }

    tree.upgraders.push(upgrader);
  }

  return tree;
}
