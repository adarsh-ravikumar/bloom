import type { MethodDeclaration } from "ts-morph";

import type { ResolverContext } from "../context";
import type { ReflectionCommand } from "../../ir/command";

import { lowerParameters } from "./parameters";

export function lowerCommand
  (ctx: ResolverContext, method: MethodDeclaration)
  : ReflectionCommand {
  return {
    name: method.getName(),
    parameters: lowerParameters(
      ctx,
      method.getParameters(),
    ),
  };
}
