import { Lua54Parser } from "./parsing/mod-parsers-wasm/pkg/index.ts";

export type ASTNode = {
  type: string;
  start: number;
  end: number;
  content?: string;
  children?: ASTNode[];
};

const lua54_parser = new Lua54Parser();

export function parseLua54(source: string): ASTNode {
  return lua54_parser.parse(source) as ASTNode;
}

export function walk(
  node: ASTNode,
  callback: (node: ASTNode, path: number[]) => void
) {
  const path: number[] = [];

  callback(node, path);

  const nodes: ASTNode[][] = [];

  if (node.children) {
    nodes.push(node.children);
    path.push(0);
  } else {
    return;
  }

  let children;

  while ((children = nodes[nodes.length - 1])) {
    const node = children[path[path.length - 1]];

    if (!node) {
      nodes.pop();
      path.pop();
      if (path.length > 0) {
        path[path.length - 1] += 1;
      }
      continue;
    }

    callback(node, path);

    if (node.children) {
      nodes.push(node.children);
      path.push(0);
    } else {
      path[path.length - 1] += 1;
    }
  }
}

export function collectTokens(node: ASTNode): string[] {
  const tokens: string[] = [];

  walk(node, (node) => {
    if (node.content != undefined) {
      tokens.push(node.content);
    }
  });

  return tokens;
}

/// Lists files
export async function findFiles(folder: string): Promise<string[]> {
  const file_list = [];
  let next_work_list = [];
  let work_list = [folder];

  while (work_list.length > 0) {
    for (const folder of work_list) {
      for await (const entry of Deno.readDir(folder)) {
        if (entry.isFile) {
          file_list.push(folder + "/" + entry.name);
        } else {
          next_work_list.push(folder + "/" + entry.name);
        }
      }
    }

    work_list = next_work_list;
    next_work_list = [];
  }

  return file_list;
}

/// Used with patch() to replace parts of a string
export class Patch {
  start: number;
  end: number;
  content: string;

  constructor(start: number, end: number, content: string) {
    this.start = start;
    this.end = end;
    this.content = content;
  }
}

/// Creates a new string with applied patches.
/// Patches should not partially overlap
export function patch(source: string, patches: Patch[]): string {
  // remove patches that would be replaced by other patches through overlap
  patches = patches.filter(
    (patch, i) =>
      !patches.some(
        (p, j) => i != j && p.start <= patch.start && p.end >= patch.end
      )
  );

  // sort the patches by start index, ascending
  patches.sort((a, b) => a.start - b.start);

  let sourceIndex = 0;
  let patchedSource = "";

  for (const patch of patches) {
    // add the content before the patch
    patchedSource += source.substring(sourceIndex, patch.start);
    // add the patch's content
    patchedSource += patch.content;
    // skip to after the patch
    sourceIndex = patch.end;
  }

  // append the remaining content
  patchedSource += source.substring(sourceIndex);

  return patchedSource;
}

/// Shallow array comparison
export function arraysEqual<T>(a: T[], b: T[]): boolean {
  return a.length == b.length && a.every((value, i) => value == b[i]);
}
