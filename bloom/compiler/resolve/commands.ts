import { lowerCommand } from "./lower/command";

import type { ResolverContext } from "./context";
import { hasDecorator } from "./widgets";

export async function resolveCommands
  (ctx: ResolverContext)
  : Promise<void> {
  for (const widget of ctx.reflection.widgets) {
    const cls = ctx.index.classes.get(widget.name);
    if (!cls) continue;

    for (const method of cls.getMethods()) {
      if (!hasDecorator(method, "Command"))
        continue;

      widget.commands.push(
        lowerCommand(ctx, method),
      );
    }
  }
}
