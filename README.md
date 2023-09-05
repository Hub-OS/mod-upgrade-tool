## Security

Upgraders only have access to `GAME_FOLDER/resources` and `./upgraders`, they can't make network requests.

## Required Software

[Deno](https://deno.land/#installation)

## Launching the upgrade tool

- Windows: `upgrade.bat`
- Linux/MacOS: `sh upgrade.sh`

With no arguments the help page will be printed:

```
Usage: upgrade.* PROJECT_ROOT START_VERSION END_VERSION
Alternate Usage: upgrade.* [OPTIONS]

This tool will overwrite files, make sure to back up your PROJECT_ROOT.

Options:
  -l, --versions-list   Lists known versions for use in START_VERSION and END_VERSION
  -h, --help
```

Example: Upgrade mods stored in `../client/mods` from 0.1 -> 0.11

- Windows: `upgrade.bat ../client 0.1 0.11`
- Linux/MacOS: `sh upgrade.sh ../client 0.1 0.11`

## Making an upgrader

Since you may be new to deno here's a [cheat sheet](https://droces.github.io/Deno-Cheat-Sheet/)

To create a new upgrader add a `*.ts` file to the `upgraders folder`, it will automatically get picked up by the upgrade tool and warn you about possible upgrade path issues.

Here's a template:

```ts
import { findFiles } from "../util.ts";

// MODIFY THESE
export const PREVIOUS_VERSION = "0.1";
export const NEXT_VERSION = "0.2";

export default async function (gameFolder: string) {
  const files = await findFiles(gameFolder + "/resources/mods");

  const luaFiles = files.filter((path) => path.toLowerCase().endsWith(".lua"));
}
```
