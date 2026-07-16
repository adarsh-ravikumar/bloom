import { parse } from "yaml";
import type { BloomContext } from "../context";

export interface ProjectConfig {
  bloom: {
    version: string,
  },
  widgets: {
    builtin: string[];
    user: string[];
  };
}


async function loadProject()
  : Promise<ProjectConfig> {

  const file =
    Bun.file("proj.bloom.yaml");

  return parse(
    await file.text(),
  ) as ProjectConfig;

}

export async function project(ctx: BloomContext): Promise<void> {
  ctx.project = await loadProject();
}
