import type { MethodDeclaration } from "ts-morph";

import type { ResolverContext } from "../context";
import type { ReflectionEvent } from "../../ir/event";

export function lowerEvent
  (_ctx: ResolverContext, method: MethodDeclaration)
  : ReflectionEvent {
  return {
    name: method.getName(),
  };
}
