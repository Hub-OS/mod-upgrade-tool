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

export function getMethodNameNode(
  functioncall_node: ASTNode
): ASTNode | undefined {
  if (functioncall_node.type != "functioncall") {
    throw new Error("not a functioncall");
  }

  const name_node =
    functioncall_node.children![functioncall_node.children!.length - 2];

  if (name_node.type != "Name") {
    return undefined;
  }

  return name_node;
}

export function getArgumentNode(
  functioncall_node: ASTNode,
  argument_index: number
): ASTNode | undefined {
  if (functioncall_node.type != "functioncall") {
    throw new Error("not a functioncall");
  }

  const args_node =
    functioncall_node.children![functioncall_node.children!.length - 1];

  const exp_list_node = args_node.children![1];

  if (!exp_list_node || exp_list_node.type != "explist") {
    return undefined;
  }

  const argument_node_index = argument_index + argument_index;

  return exp_list_node!.children![argument_node_index];
}

export function getFunctionParameters(function_node: ASTNode): ASTNode[] {
  let funcbody_node = function_node;

  if (function_node.type != "funcbody" && function_node.children) {
    funcbody_node = function_node.children[function_node.children.length - 1];
  }

  if (funcbody_node.type != "funcbody") {
    throw new Error("not a function");
  }

  const parlist_node = funcbody_node.children![1];

  if (parlist_node.type != "parlist") {
    return [];
  }

  const namelist = parlist_node.children![0];

  // skip commas
  return namelist.children!.filter((_, i) => i % 2 == 0);
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
