import { readdir } from "node:fs/promises";
import { join } from "node:path";

export async function walk(
  root: string,
): Promise<string[]> {

  const files: string[] = [];

  async function recurse(
    directory: string,
  ) {

    const entries =
      await readdir(directory, {
        withFileTypes: true,
      });

    for (const entry of entries) {

      const path =
        join(directory, entry.name);

      if (entry.isDirectory()) {
        await recurse(path);
        continue;
      }

      files.push(path);
    }
  }

  await recurse(root);

  return files;
}
