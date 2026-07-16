import type { ParameterDeclaration } from "ts-morph";

import type { ResolverContext } from "../context";
import type { ReflectionProperty } from "../../ir/property";

import { lowerType } from "./type";

export function lowerParameters
  (ctx: ResolverContext, parameters: ParameterDeclaration[])
  : ReflectionProperty[] {

  return parameters.map(parameter => ({
    name: parameter.getName(),
    optional: parameter.isOptional(),
    type: lowerType(
      ctx,
      parameter,
      parameter.getType(),
    ),
  }));
}
