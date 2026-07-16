import { Project } from "ts-morph";

import type { ResolverContext } from "./context";

export async function parse(ctx: ResolverContext): Promise<unknown> {
  ctx.project = new Project({
    tsConfigFilePath: "tsconfig.json",
  });

  ctx.sourceFiles = ctx.project.getSourceFiles();

  return;
}
