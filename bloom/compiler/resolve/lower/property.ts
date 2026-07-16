import type { PropertySignature } from "ts-morph";
import type { ReflectionProperty } from "../../ir/property";
import { lowerType } from "./type";
import type { ResolverContext } from "../context";

export function lowerProperty
  (ctx: ResolverContext, property: PropertySignature)
  : ReflectionProperty {
  return {
    name: property.getName(),
    optional: property.hasQuestionToken(),
    type: lowerType(ctx, property, property.getType()),
  };
}
