## Security

Upgraders only have access to `GAME_FOLDER/resources` and `./upgraders`, they can't make network requests.

## Required Software

[Deno](https://deno.land/#installation)

## Launching the upgrade tool

Brackets `[]` marks the value as optional, by default it will attempt to run all upgraders over the `GAME_FOLDER`.

Windows: `upgrade.bat GAME_FOLDER [FROM_VERSION] [TO_VERSION]`
Linux/MacOS: `sh upgrade.sh GAME_FOLDER [FROM_VERSION] [TO_VERSION]`

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
