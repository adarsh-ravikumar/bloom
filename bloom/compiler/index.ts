import { buildReflectionGraph } from "bloom/compiler/resolve";
import type { CompilationResult } from "./resolve/model";
import { discover } from "./discovery";
import type { BloomContext } from "scripts/lib/context";

export async function compile(ctx: BloomContext): Promise<CompilationResult> {
  const toVisit = await discover(ctx.project!);
  return await buildReflectionGraph(toVisit);
}
