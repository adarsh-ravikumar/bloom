import { basename, extname } from "node:path";
import { walk } from "./filesystem";
import type { DiscoveredWidget, WidgetSource } from "./model";

async function discoverRoot
  (root: string, source: WidgetSource)
  : Promise<DiscoveredWidget[]> {

  const widgets: DiscoveredWidget[] = [];
  const files = await walk(root);

  for (const file of files) {
    if (extname(file) !== ".ts") continue;

    widgets.push({
      file: {
        path: file,
        source,
      },

      name: basename(file, ".ts"),
    });
  }

  return widgets;
}

export async function discoverWidgets(builtin: string[], user: string[]): Promise<DiscoveredWidget[]> {
  const result = [];

  for (const b of builtin) {
    result.push(await discoverRoot(
      b, "builtin",
    ))
  }

  for (const u of user) {
    result.push(await discoverRoot(
      u, "user",
    ))
  }

  return result;
}
