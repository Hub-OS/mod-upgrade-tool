import { loadUpgradeTree } from "./upgrade_tree.ts";
import "../util.ts";

type ArgumentConfig = {
  args?: string[];
  alternate?: string;
  description?: string;
  hidden?: true;
};

const supportedArguments: { [key: string]: ArgumentConfig } = {
  "--versions-list": {
    alternate: "-l",
    description:
      "Lists known versions for use in START_VERSION and END_VERSION",
  },
  "--help": {
    alternate: "-h",
    description: "",
  },
};

async function main() {
  const flagless_args: string[] = [];
  const grouped_args = groupArgs();

  for (const arg_group of grouped_args) {
    switch (arg_group[0]) {
      case "--versions-list":
      case "-l":
        await printVersionList();
        return;
      case "--help":
      case "-h":
        printHelp();
        return;
      default:
        flagless_args.push(arg_group[0]);
    }
  }

  if (flagless_args.length < 3) {
    printHelp();
    return;
  }

  const game_folder = flagless_args[0];
  const start_version = flagless_args[1];
  const end_version = flagless_args[2];

  // initialize with defaults, automatically populated after this
  const tree = await loadUpgradeTree();

  if (!tree.upgrade(game_folder, start_version, end_version)) {
    Deno.exit(1);
  }
}

await main();

function groupArgs() {
  const processedArgs = [];
  let expectedArgs = 0;

  for (let i = 0; i < Deno.args.length; i++) {
    const arg = Deno.args[i];

    if (expectedArgs > 0) {
      processedArgs[processedArgs.length - 1].push(arg);
      expectedArgs -= 1;
      continue;
    }

    const argConfig = supportedArguments[arg];
    expectedArgs = argConfig?.args?.length || 0;

    processedArgs.push([arg]);
  }

  if (expectedArgs > 0) {
    console.error(
      "%cmissing argument for " + processedArgs[processedArgs.length - 1][0],
      "color: red"
    );
    Deno.exit(1);
  }

  return processedArgs;
}

function printHelp() {
  console.log("Usage: upgrade.* PROJECT_ROOT START_VERSION END_VERSION");
  console.log("Alternate Usage: upgrade.* [OPTIONS]\n");
  console.log(
    "This tool will overwrite files, make sure to back up your PROJECT_ROOT.\n"
  );
  console.log("Options:");

  const argsHelp = [];
  let widestHelpLength = 0;

  for (const key in supportedArguments) {
    const argConfig = supportedArguments[key];

    if (argConfig.hidden) {
      continue;
    }

    // document arg name
    const alternate = argConfig.alternate ? argConfig.alternate + "," : "   ";
    let text = "  " + alternate + " " + key + " ";

    // document arg's args
    if (argConfig.args) {
      for (const name of argConfig.args) {
        text += `<${name}> `;
      }
    }

    // store for later processing
    argsHelp.push([key, text]);

    // track largest help length
    if (text.length > widestHelpLength) {
      widestHelpLength = text.length;
    }
  }

  for (const [key, text] of argsHelp) {
    const argConfig = supportedArguments[key];
    const description = argConfig.description || "";

    console.log(text.padEnd(widestHelpLength + 2) + description);
  }
}

async function printVersionList() {
  // initialize with defaults, automatically populated after this
  const tree = await loadUpgradeTree();

  const versions: string[] = [];

  for (const upgrader of tree.upgraders) {
    if (!versions.includes(upgrader.PREVIOUS_VERSION)) {
      versions.push(upgrader.PREVIOUS_VERSION);
    }

    if (!versions.includes(upgrader.NEXT_VERSION)) {
      versions.push(upgrader.NEXT_VERSION);
    }
  }

  console.log(
    versions
      .sort((a, b) => a.localeCompare(b, undefined, { numeric: true }))
      .join("\n")
  );
}
