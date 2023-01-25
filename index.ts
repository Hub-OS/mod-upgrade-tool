// must be imported here, dynamically loading seems to break wasmoon
import "./util.ts";

type Upgrader = {
  PREVIOUS_VERSION: string;
  NEXT_VERSION: string;
  default: (game_folder: string) => Promise<void>;
};

async function main() {
  const game_folder = Deno.args[0];
  const previous_version = Deno.args[1];
  const new_version = Deno.args[2];

  if (!game_folder) {
    throw "GAME_FOLDER not specified!";
  }

  // initialize with defaults, automatically populated after this
  const version_list = ["ONB-v2"];
  const version_upgraders: { [version: string]: Upgrader } = {};

  // find upgraders
  for await (const file of Deno.readDir("upgraders")) {
    if (!file.name.toLowerCase().endsWith(".ts")) {
      // not a typescript file
      continue;
    }

    const upgrader = await import("./upgraders/" + file.name);

    if (!upgrader.PREVIOUS_VERSION) {
      throw `${file.name} is missing PREVIOUS_VERSION definition`;
    }

    if (!upgrader.default || typeof upgrader.default != "function") {
      throw `${file.name} is missing default function`;
    }

    if (version_upgraders[upgrader.PREVIOUS_VERSION]) {
      throw `multiple upgraders for ${upgrader.PREVIOUS_VERSION} exist`;
    }

    version_upgraders[upgrader.PREVIOUS_VERSION] = upgrader;
  }

  // order the upgraders
  while (true) {
    const latest_version = version_list[version_list.length - 1];
    const upgrader = version_upgraders[latest_version];

    if (!upgrader) {
      // no upgrader for the latest version, done with search
      break;
    }

    version_list.push(upgrader.NEXT_VERSION);
  }

  // make sure every upgrader has been added
  for (const key in version_upgraders) {
    const upgrader = version_upgraders[key];

    if (!version_list.includes(upgrader.PREVIOUS_VERSION)) {
      throw [
        `could not find an upgrade to ${upgrader.PREVIOUS_VERSION} `,
        `to allow for ${upgrader.PREVIOUS_VERSION} -> ${upgrader.NEXT_VERSION}`,
      ].join("");
    }
  }

  // resolve upgrade path
  const start_index = previous_version
    ? version_list.indexOf(previous_version)
    : 0;

  const latest_index = new_version
    ? version_list.indexOf(new_version)
    : version_list.length - 1;

  const end_index = latest_index - 1;

  const upgraders = [];

  for (let i = start_index; i <= end_index; i++) {
    const version = version_list[i];
    upgraders.push(version_upgraders[version]);
  }

  if (start_index == -1 || latest_index == -1) {
    console.log(
      `Could not find upgrade path from ${previous_version} to ${new_version}`
    );
    return;
  }

  console.log(
    `Upgrading from ${version_list[start_index]} to ${version_list[latest_index]}`
  );

  // time to upgrade!
  for (const upgrader of upgraders) {
    console.log(
      `Running upgrader for ${upgrader.PREVIOUS_VERSION} -> ${upgrader.NEXT_VERSION}`
    );
    await upgrader.default(game_folder);
  }
}

await main();
