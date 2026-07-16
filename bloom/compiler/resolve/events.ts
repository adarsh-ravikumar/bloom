import { lowerEvent } from "./lower/event";

import type { ResolverContext } from "./context";
import { hasDecorator } from "./widgets";

export async function resolveEvents
  (ctx: ResolverContext)
  : Promise<void> {
  for (const widget of ctx.reflection.widgets) {
    const cls = ctx.index.classes.get(widget.name);
    if (!cls) continue;

    for (const method of cls.getMethods()) {
      if (!hasDecorator(method, "Event"))
        continue;

      widget.events.push(
        lowerEvent(ctx, method),
      );
    }
  }
}
