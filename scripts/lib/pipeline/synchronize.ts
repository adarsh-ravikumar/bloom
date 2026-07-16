import { report } from "bloom/compiler/diagnostics";
import type { BloomContext } from "../context";
import { compile } from "bloom/compiler";

export async function synchronize(ctx: BloomContext) {
  ctx.compileResult = await compile(ctx);

  console.log(ctx.compileResult.reflection, { depth: null });

  if (ctx.compileResult.diagnostics.length) {
    report(ctx.logger.bloom, ctx.compileResult.diagnostics);
  }
}
